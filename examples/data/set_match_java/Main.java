import org.cprover.CProver;
public class Main {
    public static void main(int p, int s, int offset) {
        CProver.assume(p <= 3);
        Set st = new Set();

        Test.test_set(st, p, s, offset);

        int check_value = offset;
        for (int i = 0; i < p; i++) {
            int contains = st.contains(check_value);
            assert contains == 1;
            check_value =  check_value + s % 2 + 2;
        }
    }
}

