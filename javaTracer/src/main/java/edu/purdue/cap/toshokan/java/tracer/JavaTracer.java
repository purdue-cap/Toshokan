package edu.purdue.cap.toshokan.java.tracer;

import java.lang.instrument.Instrumentation;

/**
 * Hello world!
 *
 */
public class JavaTracer 
{
    public static void premain(String agentOpts, Instrumentation inst ) {
        System.out.println("Hello World with " + agentOpts);
    }
}
