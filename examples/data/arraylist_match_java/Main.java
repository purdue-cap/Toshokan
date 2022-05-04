import org.cprover.CProver;
public class Main {
    public static void main(int p, int s, int offset) {
        CProver.assume(p <= 3);
        ArrayListP list = new ArrayListP();

        Test.test_arraylist(list, p, s, offset);

        int n1 = list.get(0);
        for (int i = 1; i < p; i++) {
            int n2 = list.get(i);
            assert n2 - n1 == s % 2 + 2;
            n1 = n2;
        }
    }
}

