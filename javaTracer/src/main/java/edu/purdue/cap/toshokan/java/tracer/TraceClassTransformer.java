package edu.purdue.cap.toshokan.java.tracer;

import java.lang.instrument.ClassFileTransformer;
import java.security.ProtectionDomain;
import java.util.ArrayList;

import javassist.ClassPool;
import javassist.CtClass;
import javassist.CtMethod;
import javassist.Modifier;
import javassist.NotFoundException;

public class TraceClassTransformer implements ClassFileTransformer{
    private String targetClsName;
    private ClassLoader targetClsLoader;
    private ArrayList<String> methods;
    private boolean enableThisObj;
    @Override
    public byte[] transform(ClassLoader loader, String className, Class<?> cls, ProtectionDomain dmn, byte[] clsFile) {
        String finalClsName = this.targetClsName.replaceAll("\\.", "/");
        if (!className.equals(finalClsName)) {
            return clsFile;
        } else if (loader.equals(this.targetClsLoader)){
            CtClass cc;
            try {
                ClassPool pool = ClassPool.getDefault();
                cc = pool.get(this.targetClsName);
            } catch (NotFoundException ex) {
                System.err.println("Class not found with javassist:" + className);
                return clsFile;
            }

            for(String mtd: methods) {
                try {
                    CtMethod m = cc.getDeclaredMethod(mtd);
                    StringBuilder loggerBlock = new StringBuilder();
                    loggerBlock.append("{ com.google.gson.Gson gson = new com.google.gson.Gson();");
                    loggerBlock.append("edu.purdue.cap.toshokan.java.tracer.TraceLog log = new edu.purdue.cap.toshokan.java.tracer.TraceLog();");
                    loggerBlock.append("log.className = \""+ cc.getName() +  "\";");
                    loggerBlock.append("log.methodName = \""+ mtd +  "\";");
                    loggerBlock.append("log.args = $args;");
                    loggerBlock.append("log.ret = ($w)$_;");
                    if (!Modifier.isStatic(m.getModifiers())) {
                        if (this.enableThisObj) {
                            loggerBlock.append("log.thisObj = $0;");
                        }
                        loggerBlock.append("log.thisId = new Integer(System.identityHashCode($0));");
                    }
                    loggerBlock.append("System.err.print(\"[javaTracer] log:\");");
                    loggerBlock.append("System.err.println(gson.toJson(log));");
                    loggerBlock.append(" }");

                    m.insertAfter(loggerBlock.toString());

                } catch (Exception ex) {
                    System.err.print("Exception:");
                    System.err.println(ex);
                }
            }
            try {
                byte[] newClsFile = cc.toBytecode();
                cc.detach();
                return newClsFile;
            } catch (Exception ex) {
                System.err.print("Exception:");
                System.err.println(ex);
            }
        }
        return clsFile;
    }
    public TraceClassTransformer(String className, ClassLoader clsLd, ArrayList<String> methods, boolean enableThisObj) {
        this.targetClsName = className;
        this.targetClsLoader = clsLd;
        this.methods = methods;
        this.enableThisObj = enableThisObj;
    }
}
