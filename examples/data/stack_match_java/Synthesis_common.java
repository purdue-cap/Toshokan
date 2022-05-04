@JavaCodeGen
public class Test {
    public static void test_stack(Stack st, int p, int s, int offset) {
        int adder = ??;
        int factor = ??;
        for (int i = 0; i < p; i++) {
            st.push(i * (s % factor + adder) + offset);
        }
    }
}