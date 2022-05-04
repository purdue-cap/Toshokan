@JavaCodeGen
public class Test {
    public static void test_arraylist(ArrayListP list, int p, int s, int offset) {
        int adder = ??;
        int factor = ??;
        for (int i = 0; i < p; i++) {
            list.push_back(i * (s % factor + adder) + offset);
        }
    }
}