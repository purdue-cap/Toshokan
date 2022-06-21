package edu.purdue.cap.toshokan.java.tracer;

import java.lang.instrument.Instrumentation;
import java.util.HashMap;
import java.util.Map;
import java.util.ArrayList;

import com.google.gson.Gson;

/**
 * Hello world!
 *
 */
public class JavaTracer 
{
    public static void premain(String agentOpts, Instrumentation inst ) {
        Gson gson = new Gson();
        JavaTracerConfig config = gson.fromJson(agentOpts, JavaTracerConfig.class);
        HashMap<String, ArrayList<String>> mtdPerCls = new HashMap<>();
        for (JavaTracerConfig.MethodInfo info : config.methods) {
            if (!mtdPerCls.containsKey(info.className)) {
                mtdPerCls.put(info.className, new ArrayList<String>());
            }
            mtdPerCls.get(info.className).add(info.method);
        }
        for (Map.Entry<String, ArrayList<String>> entry: mtdPerCls.entrySet()) {
            transformClass(entry.getKey(), entry.getValue(), inst);
        }
        
    }

    private static void transformClass(String className, ArrayList<String> methods, Instrumentation inst) {
        Class<?> targetCls;
        ClassLoader targetClLd;
        try {
            targetCls = Class.forName(className);
            targetClLd = targetCls.getClassLoader();
            transform(targetCls, targetClLd, methods, inst);
            return;
        } catch (Exception exp) {
            System.err.println("[javaTracer] Class not found with Class.forName");
        }
        for (Class<?> cl : inst.getAllLoadedClasses()) {
            if (cl.getName().equals(className)) {
                targetCls = cl;
                targetClLd = targetCls.getClassLoader();
                transform(targetCls, targetClLd, methods, inst);
                return;
            }
        }
        throw new RuntimeException("Failed to find class:" + className);
    }

    private static void transform(Class<?> cl, ClassLoader loader, ArrayList<String> methods, Instrumentation inst) {
        TraceClassTransformer tr = new TraceClassTransformer(cl.getName(), loader, methods);
        inst.addTransformer(tr, true);
        try {
            inst.retransformClasses(cl);
        } catch (Exception ex) {
            throw new RuntimeException("Transform failed for:"+ cl.getName(), ex);
        }
    }
}
