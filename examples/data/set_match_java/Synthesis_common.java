@JavaCodeGen
public class Test {
    public static void test_set(Set st, int p, int s, int offset) {
        int adder = ??;
        int factor = ??;
        for (int i = 0; i < p; i++) {
            st.add(i * (s % factor + adder) + offset);
        }
    }
}