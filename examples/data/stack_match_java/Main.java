import org.cprover.CProver;
public class Main {
    public static void main(int p, int s, int offset) {
        CProver.assume(p <= 3);
        Stack st = new Stack();

        Test.test_stack(st, p, s, offset);

        int last_pop = st.pop();
        for (int i = 0; i < p - 1; i++) {
            int new_pop = st.pop();
            assert new_pop + (s % 2 + 2) == last_pop;
            last_pop = new_pop;
        }
    }
}

