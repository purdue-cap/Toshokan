#include <string>
#include <sstream>
#include <vector>
#include <system_error>

#include "clang/AST/AST.h"
#include "clang/AST/ASTConsumer.h"
#include "clang/AST/RecursiveASTVisitor.h"
#include "clang/Frontend/CompilerInstance.h"
#include "clang/Frontend/FrontendAction.h"
#include "clang/Rewrite/Core/Rewriter.h"
#include "clang/Tooling/Tooling.h"
#include "clang/Tooling/CompilationDatabase.h"

#include "inja.hpp"


namespace TracerBuilder {

using namespace clang;
using namespace clang::tooling;

using json = nlohmann::json;

using std::string;
using std::stringstream;
using std::vector;

class TracerBuilderVisitor : public RecursiveASTVisitor<TracerBuilderVisitor> {
    public:
    TracerBuilderVisitor(Rewriter &R, string &name) : TheRewriter(R), LibFuncName(name) {}

    bool VisitFunctionDecl(FunctionDecl *f) {
        auto &SM = TheRewriter.getSourceMgr();
        if (f->hasBody()
            && f->getNameAsString() == LibFuncName
            && SM.isInMainFile(f->getBeginLoc())) {
            // Construct impl function name
            stringstream impl_name;
            impl_name << LibFuncName << "_impl";

            // Build actual parameter list
            vector<string> arg_name_list;
            vector<string> arg_type_list;
            for (unsigned i = 0; i < f->getNumParams() - 1; i++) {
                arg_name_list.push_back(f->getParamDecl(i)->getName().str());
                arg_type_list.push_back(f->getParamDecl(i)->getType().getAsString());
            }

            // Get the returning paramter
            const auto* rtn_arg = f->getParamDecl(f->getNumParams() - 1);
            auto rtn_arg_name = rtn_arg->getName().str();
            auto rtn_type_name = rtn_arg->getType().getNonReferenceType().getAsString();

            json data;
            data["lib_func_name"] = LibFuncName;
            data["arg_list"] = arg_name_list;
            data["rtn_arg"] = rtn_arg_name;
            data["arg_list_rendered"] = inja::render(
                R"({% for arg in arg_list %}{{ arg }}{% if not loop.is_last %}, {% endif %}{% endfor %})",
                data
            );
            data["arg_fmt_rendered"] = inja::render(
                R"({% for arg in arg_list %}%d{% if not loop.is_last %}, {% endif %}{% endfor %})",
                data
            );
            data["arg_types"] = arg_type_list;
            data["rtn_type"] = rtn_type_name;
            data["arg_types_rendered"] = inja::render(
                R"({% for type in arg_types %}{{ type }}{% if not loop.is_last %}, {% endif %}{% endfor %})",
                data
            );
            string body_template(
R"({
    {{ rtn_arg }} = {{ lib_func_name }}_impl({{ arg_list_rendered }});
    fprintf("{{ lib_func_name }}({{ arg_fmt_rendered }}) = %d", {{ arg_list_rendered }}, {{ rtn_arg }});
}
)"
            );
            string decl_template(
R"(
{{ rtn_type }} {{ lib_func_name }}_impl({{ arg_types_rendered }});
)"
            );

            auto range = f->getBody()->getSourceRange();
            TheRewriter.ReplaceText(range, inja::render(body_template, data));
            auto startOfFile = SM.getLocForStartOfFile(SM.getMainFileID());
            TheRewriter.InsertText(startOfFile, inja::render(decl_template, data));
        }
        return true;
    }

    private:
    Rewriter &TheRewriter;
    string &LibFuncName;
};

class TracerBuilderASTConsumer : public ASTConsumer {
    public:
    TracerBuilderASTConsumer(Rewriter &R, string &name) : Visitor(R, name) {}

    // Override the method that gets called for each parsed top-level
    // declaration.
    bool HandleTopLevelDecl(DeclGroupRef DR) override {
        for (auto b = DR.begin(), e = DR.end(); b != e; ++b) {
            // Traverse the declaration using our AST visitor.
            Visitor.TraverseDecl(*b); }
        return true;
    }

    private:
    TracerBuilderVisitor Visitor;
};

class TracerBuilderFrontendAction : public ASTFrontendAction {
    public:
    TracerBuilderFrontendAction(string &name, llvm::raw_ostream &out): LibFuncName(name), OutStream(out) {}

    void EndSourceFileAction() override {
        SourceManager &SM = TheRewriter.getSourceMgr();

        // Now emit the rewritten buffer.
        TheRewriter.getEditBuffer(SM.getMainFileID()).write(OutStream);
    }

    std::unique_ptr<ASTConsumer> CreateASTConsumer(CompilerInstance &CI,
                                                    StringRef file) override {
        TheRewriter.setSourceMgr(CI.getSourceManager(), CI.getLangOpts());
        return std::make_unique<TracerBuilderASTConsumer>(TheRewriter, LibFuncName);
    }

    private:
    Rewriter TheRewriter;
    string &LibFuncName;
    llvm::raw_ostream &OutStream;
};

class TracerBuilderFrontendActionFactory : public FrontendActionFactory {
    public:
    TracerBuilderFrontendActionFactory(string &name, llvm::raw_ostream &out) : LibFuncName(name), OutStream(out) {}

    std::unique_ptr<FrontendAction> create() {
        return  std::make_unique<TracerBuilderFrontendAction>(LibFuncName, OutStream);
    }

    private:
    string LibFuncName;
    llvm::raw_ostream &OutStream;
};

int BuildTracer(string& lib_func_name, string& input_file, llvm::raw_ostream &out) {
    string source_path_list[1] = {input_file};
    auto dir_path = llvm::sys::path::parent_path(input_file);
    llvm::outs() << dir_path;
    string error_msg;
    auto db = CompilationDatabase::loadFromDirectory(dir_path, error_msg);
    if (db == nullptr) {
        return 1;
    }
    ClangTool tool(*db, source_path_list);
    TracerBuilderFrontendActionFactory factory(lib_func_name, out);
    return tool.run(&factory);
}

int BuildTracer(string& lib_func_name, string& input_file, string& output_file) {
    std::error_code EC;
    llvm::raw_fd_ostream out_stream(output_file, EC);
    if (EC) {
        return EC.value();
    }
    return BuildTracer(lib_func_name, input_file, out_stream);
}

} // namespace TracerBuilder


// C version of the functions

extern "C" {

int build_tracer(const char* lib_func_name, const char* input_file, const char* output_file){
    std::string LibFuncName(lib_func_name);
    std::string InputFile(input_file);
    std::string OutputFile(output_file);
    return TracerBuilder::BuildTracer(LibFuncName, InputFile, OutputFile);
}

}

#ifdef TESTBIN
using namespace TracerBuilder;

int main(int argc, const char **argv) {
    if (argc < 3) {
        return 1;
    }
    string input_file(argv[1]);
    string func_name(argv[2]);

    if (argc >= 4) {
        string output_file(argv[3]);
        return BuildTracer(func_name, input_file, output_file);
    } else {
        return BuildTracer(func_name, input_file, llvm::outs());
    }

}
#endif // TESTBIN