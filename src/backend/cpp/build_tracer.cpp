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

#ifdef TESTBIN
#include "clang/Tooling/CommonOptionsParser.h"
#endif // TESTBIN

namespace TracerBuilder {

using namespace clang;
using namespace clang::tooling;

using std::string;
using std::stringstream;
using std::vector;

#ifdef TESTBIN
static llvm::cl::OptionCategory TestBinCategory("Test Binary");
#endif // TESTBIN

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
            stringstream arg_list;
            for (unsigned i = 0; i < f->getNumParams() - 1; i++) {
                arg_list << f->getParamDecl(i)->getName().str();
                if (i != f->getNumParams() - 2) {
                    arg_list << ", ";
                }
            }

            // Build impl function call
            stringstream impl_call;
            impl_call << impl_name.str() << "(" << arg_list.str() << ")";

            // Get the returning paramter
            const auto* rtn_param = f->getParamDecl(f->getNumParams() - 1);

            // Construct log str
            stringstream log_str;
            log_str << '"' << LibFuncName << "(";
            for (unsigned i = 0; i < f->getNumParams() - 1; i++) {
                log_str << "%d"; // TODO: Support more types than integer here
                if (i != f->getNumParams() - 2) {
                    log_str << ", ";
                }
            }
            log_str << ") = %d\\n" << '"';

            // Construct log statement
            stringstream log_statement;
            log_statement << "fprintf(stderr, " << log_str.str() << ", "
            << arg_list.str() << ", " << rtn_param->getName().str() << ")";


            // Build Body
            stringstream body;
            body << "{\n"
            << "  " << rtn_param->getName().str() << " = " << impl_call.str() << ";\n"
            << "  " << log_statement.str() << ";\n"
            << "}";

            // Construct declaration
            stringstream decl;
            decl << rtn_param->getType().getNonReferenceType().getAsString()
            << " " << impl_name.str() << "(";
            for (unsigned i = 0; i < f->getNumParams() - 1; i++) {
                decl << f->getParamDecl(i)->getType().getAsString();
                if (i != f->getNumParams() - 2) {
                    decl << ", ";
                }
            }
            decl << ");\n";


            auto range = f->getBody()->getSourceRange();
            TheRewriter.ReplaceText(range, body.str());
            auto startOfFile = SM.getLocForStartOfFile(SM.getMainFileID());
            TheRewriter.InsertText(startOfFile, decl.str());
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
            Visitor.TraverseDecl(*b);
        }
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
        return llvm::make_unique<TracerBuilderASTConsumer>(TheRewriter, LibFuncName);
    }

    private:
    Rewriter TheRewriter;
    string &LibFuncName;
    llvm::raw_ostream &OutStream;
};

class TracerBuilderFrontendActionFactory : public FrontendActionFactory {
    public:
    TracerBuilderFrontendActionFactory(string &name, llvm::raw_ostream &out) : LibFuncName(name), OutStream(out) {}

    FrontendAction* create() {
        return new TracerBuilderFrontendAction(LibFuncName, OutStream);
    }

    private:
    string LibFuncName;
    llvm::raw_ostream &OutStream;
};

int BuildTracer(string& lib_func_name, string& input_file, llvm::raw_ostream &out) {
    string source_path_list[1] = {input_file};
    auto dir_path = llvm::sys::path::parent_path(input_file);
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