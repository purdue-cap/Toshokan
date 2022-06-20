package edu.purdue.cap.toshokan.java.tracer;

import java.lang.instrument.Instrumentation;
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
        System.out.println("Hello World with " + config.info);
    }
}
