#include "clang/AST/AST.h"
#include "clang/AST/ASTConsumer.h"
#include "clang/AST/RecursiveASTVisitor.h"
#include "clang/Frontend/CompilerInstance.h"
#include "clang/Frontend/FrontendAction.h"
#include "clang/Rewrite/Core/Rewriter.h"
#include "clang/Tooling/Tooling.h"

#ifdef TESTBIN
#include "clang/Tooling/CommonOptionsParser.h"
#endif // TESTBIN

namespace TracerBuilder {

using namespace clang;
using namespace clang::tooling;

#ifdef TESTBIN
static llvm::cl::OptionCategory TestBinCategory("Test Binary");
#endif // TESTBIN

class TracerBuilderVisitor : public RecursiveASTVisitor<TracerBuilderVisitor> {
    public:
    TracerBuilderVisitor(Rewriter &R) : TheRewriter(R) {}

    private:
    Rewriter &TheRewriter;
};

class TracerBuilderASTConsumer : public ASTConsumer {
    public:
    TracerBuilderASTConsumer(Rewriter &R) : Visitor(R) {}

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
    TracerBuilderFrontendAction() {}

    void EndSourceFileAction() override {
        SourceManager &SM = TheRewriter.getSourceMgr();
        llvm::errs() << "** EndSourceFileAction for: "
                    << SM.getFileEntryForID(SM.getMainFileID())->getName() << "\n";

        // Now emit the rewritten buffer.
        TheRewriter.getEditBuffer(SM.getMainFileID()).write(llvm::outs());
    }

    std::unique_ptr<ASTConsumer> CreateASTConsumer(CompilerInstance &CI,
                                                    StringRef file) override {
        llvm::errs() << "** Creating AST consumer for: " << file << "\n";
        TheRewriter.setSourceMgr(CI.getSourceManager(), CI.getLangOpts());
        return llvm::make_unique<TracerBuilderASTConsumer>(TheRewriter);
    }

    private:
    Rewriter TheRewriter;
};

} // namespace TracerBuilder

#ifdef TESTBIN
using namespace TracerBuilder;

int main(int argc, const char **argv) {
    CommonOptionsParser op(argc, argv, TestBinCategory);
    ClangTool tool(op.getCompilations(), op.getSourcePathList());

    return tool.run(newFrontendActionFactory<TracerBuilderFrontendAction>().get());
}
#endif // TESTBIN