#include <string>
#include <sstream>
#include <vector>
#include <unordered_set>
#include <system_error>

#include "clang/AST/AST.h"
#include "clang/AST/ASTConsumer.h"
#include "clang/AST/Comment.h"
#include "clang/AST/RecursiveASTVisitor.h"
#include "clang/ASTMatchers/ASTMatchers.h"
#include "clang/ASTMatchers/ASTMatchFinder.h"
#include "clang/Frontend/CompilerInstance.h"
#include "clang/Frontend/FrontendAction.h"
#include "clang/Rewrite/Core/Rewriter.h"
#include "clang/Tooling/Tooling.h"
#include "clang/Tooling/CompilationDatabase.h"

#include "inja.hpp"


namespace TracerBuilder {

using namespace clang;
using namespace clang::tooling;
using namespace clang::ast_matchers;

using json = nlohmann::json;

using std::string;
using std::stringstream;
using std::vector;
using std::unordered_set;
using string_set = unordered_set<string>;

QualType getBaseType(QualType type) {
    auto base_type = type.getNonReferenceType();
    while (base_type->isPointerType()) {
        base_type = base_type->getPointeeType();
    }
    return base_type;
}

const StatementMatcher copyarr_out_call_matcher =
callExpr(
    anyOf(
        allOf(
            callee(namedDecl(hasName("CopyArr"))),
            hasArgument(0, declRefExpr(to(namedDecl(hasName("_out"))))),
            hasArgument(1, integerLiteral(equals(0))),
            hasArgument(2, integerLiteral().bind("lenLiteral"))
        ),
        allOf(
            callee(namedDecl(hasName("CopyArr"))),
            hasArgument(0, declRefExpr(to(namedDecl(hasName("_out"))))),
            hasArgument(1, declRefExpr()),
            hasArgument(2, integerLiteral()),
            hasArgument(3, integerLiteral().bind("lenLiteral"))
        )
    )
).bind("copyArrCall");

class CopyArrLookUpCallback : public MatchFinder::MatchCallback {
    public: 
    CopyArrLookUpCallback() : foundResult(nullptr), arrayLengthStr() {}
    void run(const MatchFinder::MatchResult& result) {
        foundResult = result.Nodes.getNodeAs<CallExpr>("copyArrCall");
        arrayLengthStr = result.Nodes.getNodeAs<IntegerLiteral>("lenLiteral")->getValue().toString(10, true);
    }

    const CallExpr* getFoundResult() {return foundResult;}
    const string& getArrayLengthStr(){return arrayLengthStr;}

    private:
    const CallExpr* foundResult;
    string arrayLengthStr;

};


class ImplFuncInjector : public RecursiveASTVisitor<ImplFuncInjector> {
    public:
    ImplFuncInjector(Rewriter &R, string &name, string_set states = string_set())
        : TheRewriter(R), LibFuncName(name), StateArgNames(std::move(states)), printingPolicy(LangOptions()) {
            printingPolicy.SuppressTagKeyword = true;
        }

    bool VisitFunctionDecl(FunctionDecl *f) {
        auto &SM = TheRewriter.getSourceMgr();
        if (f->hasBody()
            && f->getNameAsString() == LibFuncName
            && SM.isInMainFile(f->getBeginLoc())) {

            vector<string> arg_name_list;
            vector<string> arg_type_list;
            vector<string> arg_decl_list;
            const ParmVarDecl* rtn_arg = nullptr;
            vector<const ParmVarDecl*> state_args;
            for (const auto* arg : f->parameters()) {
                auto arg_name = arg->getName().str();
                if (arg_name == "_out") {
                    rtn_arg = arg;
                    continue;
                }
                if (arg_name.find("__ANONYMOUS_") != string::npos) {
                    continue;
                }
                auto arg_type = arg->getType();
                auto base_type = getBaseType(arg_type);

                if (!buildJSONConversionForType(base_type)) {
                    return false;
                }

                if (StateArgNames.find(arg_name) != StateArgNames.end()) {
                    state_args.push_back(arg);
                }

                arg_name_list.push_back(arg_name);
                auto type_name = arg_type.getAsString(printingPolicy);
                // Workaround for resolving _Bool to bool
                if (type_name == "_Bool") {
                    type_name = "bool";
                }
                arg_type_list.push_back(type_name);
            }

            // Get the returning paramter
            auto rtn_arg_name = rtn_arg->getName().str();
            auto rtn_type = rtn_arg->getType().getNonReferenceType();

            bool out_is_array = rtn_type->isPointerType();
            string array_length_str;

            if (out_is_array) {
                // Returning an array, looking for CopyArr call to get array length
                CopyArrLookUpCallback callback;
                MatchFinder finder;
                DeclarationMatcher subtree_matcher = hasDescendant(copyarr_out_call_matcher);
                finder.addMatcher(subtree_matcher, &callback);
                finder.match(*f, f->getASTContext());
                if (callback.getFoundResult() == nullptr) {
                    return false;
                }
                array_length_str = callback.getArrayLengthStr();

                rtn_type = rtn_type->getPointeeType();

            }

            // Returning type must be traversed for JSON conversion as well
            if (!buildJSONConversionForType(rtn_type)) {
                return false;
            }

            auto rtn_type_name = rtn_type.getAsString(printingPolicy);

            // Workaround for resolving _Bool to bool
            if (rtn_type_name == "_Bool") {
                rtn_type_name = "bool";
            }

            if (out_is_array) {
                rtn_type_name = "std::vector<" + rtn_type_name + ">";
            }

            json data;
            data["lib_func_name"] = LibFuncName;
            data["arg_list"] = arg_name_list;
            data["rtn_arg"] = rtn_arg_name;
            data["arg_list_rendered"] = inja::render(
                R"({% for arg in arg_list %}{{ arg }}{% if not loop.is_last %}, {% endif %}{% endfor %})",
                data
            );
            data["arg_types"] = arg_type_list;
            data["rtn_type"] = rtn_type_name;
            data["arg_types_rendered"] = inja::render(
                R"({% for type in arg_types %}{{ type }}{% if not loop.is_last %}, {% endif %}{% endfor %})",
                data
            );

            if (!state_args.empty()) {
                data["state_arg_names"] = StateArgNames;
                data["state_log_stmt_rendered"] = inja::render(
R"(
    json updated_states;{% for state in state_arg_names %}
    updated_states["{{ state }}"] = {{ state }};{% endfor %}
    log["updated_states"] = updated_states;
)",
                    data
                );
            } else {
                data["state_log_stmt_rendered"] = "";
            }

            string body_template;

            if (out_is_array) {
                data["array_length"] = array_length_str;
                body_template = 
R"({
    {{ rtn_type }} rtn_vec = {{ lib_func_name }}_impl({{ arg_list_rendered }});
    for (int i = 0; i < {{ array_length }}; i++) {
        {{ rtn_arg }}[i] = rtn_vec[i];
    }
    json log;
    std::vector<json> args;{% for arg in arg_list %}
    args.push_back(json({{ arg }}));{% endfor %}
    log["args"] = args;
    log["rtn"] = rtn_vec;
    {{ state_log_stmt_rendered }}
    std::cerr << log << std::endl;
}
)";
            } else {
                body_template = 
R"({
    {{ rtn_arg }} = {{ lib_func_name }}_impl({{ arg_list_rendered }});
    json log;
    std::vector<json> args;{% for arg in arg_list %}
    args.push_back(json({{ arg }}));{% endfor %}
    log["args"] = args;
    log["rtn"] = {{ rtn_arg }};
    {{ state_log_stmt_rendered }}
    std::cerr << log << std::endl;
}
)";
            }
            string decl_template(
R"(
{{ rtn_type }} {{ lib_func_name }}_impl({{ arg_types_rendered }});
)"
            );

            auto range = f->getBody()->getSourceRange();
            TheRewriter.ReplaceText(range, inja::render(body_template, data));

            DeclContext* outer_context = f;
            while(!outer_context->getLexicalParent()->isTranslationUnit()) {
                outer_context = outer_context->getLexicalParent();
            }
            auto* outer_decl = dyn_cast<NamespaceDecl>(outer_context);
            auto insert_loc = outer_decl->getBeginLoc();

            string json_include = "#include \"json.hpp\"\n";
            string vector_include = "#include <vector>\n";
            string iostream_include = "#include <iostream>\n";
            string json_use = "using nlohmann::json;\n";

            TheRewriter.InsertText(insert_loc, json_include);
            TheRewriter.InsertText(insert_loc, vector_include);
            TheRewriter.InsertText(insert_loc, iostream_include);
            TheRewriter.InsertText(insert_loc, json_use);
            for (auto decl_str: JSONConvertorDecls) {
                TheRewriter.InsertText(insert_loc, decl_str);
            }
            for (auto impl_str: JSONConvertorImpls) {
                TheRewriter.InsertText(insert_loc, impl_str);
            }
            TheRewriter.InsertText(insert_loc, inja::render(decl_template, data));
        }
        return true;
    }

    private:
    Rewriter &TheRewriter;
    string &LibFuncName;
    string_set StateArgNames;
    vector<string> JSONConvertorDecls;
    vector<string> JSONConvertorImpls;
    unordered_set<string> doneType;
    PrintingPolicy printingPolicy;
    bool buildJSONConversionForType(QualType type){
        auto type_name = type.getUnqualifiedType().getAsString(printingPolicy);
        if (type->isFundamentalType() || 
            doneType.find(type_name) != doneType.end()) {
            // It's fundamental type, or it is already traversed
            return true;
        }
        doneType.insert(type_name);
        bool is_array = type_name.find("array::Array_") != string::npos;

        const auto* record_decl = type->getAsRecordDecl();
        if (record_decl == nullptr) {
            // type is non-record non-fundamental type, we can't handle it at the moment
            return false;
        }

        vector<string> field_names;
        string array_element_type_name;

        if (is_array) {
            // Make sure this is a sketch array
            bool length_found = false;
            bool A_found = false;
            QualType A_type;
            for (auto field: record_decl->fields()) {
                if (field->getNameAsString() == "length" && field->getType()->isIntegerType()) {
                    length_found = true;
                }
                if (field->getNameAsString() == "A" && field->getType()->isArrayType()) {
                    A_found = true;
                    A_type = field->getType();
                }
            }
            if (!length_found || !A_found) {
                return false;
            }
            // Traverse its base type
            auto array_type = record_decl->getASTContext().getAsArrayType(A_type);
            auto element_type = array_type->getElementType();
            array_element_type_name = element_type.getUnqualifiedType().getAsString(printingPolicy);
            auto base_type = getBaseType(element_type);
            if (!buildJSONConversionForType(base_type)) {
                return false;
            }
        } else {
            // Traverse a non-array struct 
            for (auto field: record_decl->fields()) {
                auto base_type = getBaseType(field->getType());
                if (!buildJSONConversionForType(base_type)) {
                    return false;
                }
                field_names.push_back(field->getNameAsString());
            }
        }
        json template_data;
        template_data["type_name"] = type_name;
        template_data["field_names"] = field_names;

        // Workaround for resolving _Bool to bool
        if (array_element_type_name == "_Bool") {
            array_element_type_name = "bool";
        }
        template_data["array_element_type_name"] = array_element_type_name;

        string decl_template(
R"(
template<>
struct nlohmann::adl_serializer<{{ type_name }}>{
    static void to_json(json &, const {{ type_name }}&);
};
template<>
struct nlohmann::adl_serializer<{{ type_name }}*>{
    static void to_json(json &, const {{ type_name }}*);
};
)"
        );

        string impl_template;
        if (is_array) {
            impl_template =
R"(
void nlohmann::adl_serializer<{{ type_name }}>::to_json(json &j, const {{ type_name }} &data){
    std::vector<{{ array_element_type_name }}> vec;
    for (int i; i < data.length; i++) {
        vec.push_back(data.A[i]);
    }
    j = vec;
}
void nlohmann::adl_serializer<{{ type_name }}*>::to_json(json &j, const {{ type_name }} *data){
    if (data == nullptr) {
        j = nullptr;
    } else {
        j = *data;
    }
}
)";
        } else {
            impl_template =
R"(
void nlohmann::adl_serializer<{{ type_name }}>::to_json(json &j, const {{ type_name }} &data){
    j = { {% for field_name in field_names %}
        {"{{ field_name }}", data.{{ field_name }} }{%if not loop.is_last%},{% endif %}{% endfor %}
    };
}
void nlohmann::adl_serializer<{{ type_name }}*>::to_json(json &j, const {{ type_name }} *data){
    if (data == nullptr) {
        j = nullptr;
    } else {
        j = { {% for field_name in field_names %}
            {"{{ field_name }}", data->{{ field_name }} }{%if not loop.is_last%},{% endif %}{% endfor %}
        };
    }
}
)";
        }

        JSONConvertorDecls.push_back(inja::render(decl_template, template_data));
        JSONConvertorImpls.push_back(inja::render(impl_template, template_data));

        return true;
    }
};

const StatementMatcher assumption_removal_matcher =
ifStmt(hasThen(has(
    exprWithCleanups(has(
        cxxThrowExpr(has(
            cxxConstructExpr(
                hasDeclaration(cxxConstructorDecl(
                    hasName("AssumptionFailedException")
                ))
            )
        ))
    ))
))).bind("stmtToBeRemoved");

const StatementMatcher assertion_removal_matcher = 
parenExpr(has(
    conditionalOperator(hasFalseExpression(
        callExpr(
            callee(functionDecl(
                hasName("__assert_fail")
            ))
        )
    ))
)).bind("stmtToBeRemoved");

const DeclarationMatcher using_directive_matcher = 
usingDirectiveDecl(
).bind("usingDirective");

class StmtCleanUpCallback : public MatchFinder::MatchCallback {
    public: 
    StmtCleanUpCallback(Rewriter &R) : TheRewriter(R) {}
    void run(const MatchFinder::MatchResult& result) {
        const Stmt* stmt = result.Nodes.getNodeAs<Stmt>("stmtToBeRemoved");
        auto &SM = TheRewriter.getSourceMgr();

        auto begin_loc = stmt->getBeginLoc();
        auto end_loc = stmt->getEndLoc();
        SourceRange range(begin_loc, end_loc);
        if (begin_loc.isMacroID()) {
            auto expansion_range = SM.getImmediateExpansionRange(begin_loc);
            range = expansion_range.getAsRange();
        }

        if (SM.isInMainFile(range.getBegin())) {
            TheRewriter.RemoveText(range);
        }
    }
    private:
    Rewriter &TheRewriter;
};

class UsingDirectiveCleanUpCallback : public MatchFinder::MatchCallback {
    public: 
    UsingDirectiveCleanUpCallback(Rewriter &R) : TheRewriter(R) {}
    void run(const MatchFinder::MatchResult& result) {
        const UsingDirectiveDecl* decl = result.Nodes.getNodeAs<UsingDirectiveDecl>("usingDirective");
        auto ctx = decl->getDeclContext();
        if (!ctx) {
            return;
        }
        if (!ctx->isTranslationUnit()) {
            return;
        }
        auto &SM = TheRewriter.getSourceMgr();

        auto begin_loc = decl->getBeginLoc();
        auto end_loc = decl->getEndLoc();
        SourceRange range(begin_loc, end_loc);
        if (begin_loc.isMacroID()) {
            auto expansion_range = SM.getImmediateExpansionRange(begin_loc);
            range = expansion_range.getAsRange();
        }

        if (SM.isInMainFile(range.getBegin())) {
            TheRewriter.RemoveText(range);
        }
    }
    private:
    Rewriter &TheRewriter;
};

class TracerBuilderASTConsumer : public ASTConsumer {
    public:
    TracerBuilderASTConsumer(Rewriter &R, string &name, bool &done, string_set state = string_set())
        : impl_func_injector(R, name, std::move(state)), stmt_cleanup_cb(R), using_cleanup_cb(R), done(done) {}

    // Override the method that gets called for each parsed top-level
    // declaration.
    bool HandleTopLevelDecl(DeclGroupRef DR) override {
        for (auto b = DR.begin(), e = DR.end(); b != e; ++b) {
            if (!impl_func_injector.TraverseDecl(*b)) {
                done = false;
                return false;
            }
        }
        MatchFinder finder;
        finder.addMatcher(assumption_removal_matcher, &stmt_cleanup_cb);
        finder.addMatcher(assertion_removal_matcher, &stmt_cleanup_cb);
        finder.addMatcher(using_directive_matcher, &using_cleanup_cb);
        for (auto b = DR.begin(), e = DR.end(); b != e; ++b) {
            finder.matchAST((*b)->getASTContext());
        }
        done = true;
        return true;
    }

    private:
    ImplFuncInjector impl_func_injector;
    StmtCleanUpCallback stmt_cleanup_cb;
    UsingDirectiveCleanUpCallback using_cleanup_cb;
    bool& done;
};

class TracerBuilderFrontendAction : public ASTFrontendAction {
    public:
    TracerBuilderFrontendAction(string &name, llvm::raw_ostream &out, bool& done, string_set states = string_set())
        : LibFuncName(name), StateArgNames(std::move(states)), OutStream(out), done(done) {}

    void EndSourceFileAction() override {
        SourceManager &SM = TheRewriter.getSourceMgr();

        // Now emit the rewritten buffer.
        TheRewriter.getEditBuffer(SM.getMainFileID()).write(OutStream);
    }

    std::unique_ptr<ASTConsumer> CreateASTConsumer(CompilerInstance &CI,
                                                    StringRef file) override {
        TheRewriter.setSourceMgr(CI.getSourceManager(), CI.getLangOpts());
        return std::make_unique<TracerBuilderASTConsumer>(TheRewriter, LibFuncName, done, StateArgNames);
    }

    private:
    Rewriter TheRewriter;
    string &LibFuncName;
    string_set StateArgNames;
    llvm::raw_ostream &OutStream;
    bool& done;
};

class TracerBuilderFrontendActionFactory : public FrontendActionFactory {
    public:
    TracerBuilderFrontendActionFactory(string &name, llvm::raw_ostream &out, bool& done, string_set states = string_set())
        : LibFuncName(name), StateArgNames(std::move(states)), OutStream(out), done(done) {}

    std::unique_ptr<FrontendAction> create() {
        return  std::make_unique<TracerBuilderFrontendAction>(LibFuncName, OutStream, done, StateArgNames);
    }

    private:
    string LibFuncName;
    string_set StateArgNames;
    llvm::raw_ostream &OutStream;
    bool& done;
};

int BuildTracer(string& lib_func_name, string& input_file, llvm::raw_ostream &out, string_set states = string_set()) {
    string source_path_list[1] = {input_file};
    auto dir_path = llvm::sys::path::parent_path(input_file);
    string error_msg;
    auto db = CompilationDatabase::loadFromDirectory(dir_path, error_msg);
    if (db == nullptr) {
        llvm::errs() << error_msg;
        return 1;
    }
    ClangTool tool(*db, source_path_list);
    bool done;
    TracerBuilderFrontendActionFactory factory(lib_func_name, out, done, std::move(states));
    int rtn_code = tool.run(&factory);
    if (rtn_code != 0){
        return rtn_code;
    } else if (done) {
        return 0;
    } else {
        return 3;
    }
}

int BuildTracer(string& lib_func_name, string& input_file, string& output_file, string_set states = string_set()) {
    std::error_code EC;
    llvm::raw_fd_ostream out_stream(output_file, EC);
    if (EC) {
        return EC.value();
    }
    return BuildTracer(lib_func_name, input_file, out_stream, std::move(states));
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
        string_set states;
        for (int i = 3; i < argc; i ++) {
            states.insert(string(argv[i]));
        }
        return BuildTracer(func_name, input_file, llvm::outs(), std::move(states));
    } else {
        return BuildTracer(func_name, input_file, llvm::outs());
    }

}
#endif // TESTBIN