@rewriteClass
class ArrayListP {
    @alg
    Object pushback(int e);

    @alg
    @pure
    int get(int i);

    @alg
    @pure
    int size();

    rewrite Object size(Object ArrayListP()) {
        return 0;
    }

    rewrite Object size(Object pushback!(ArrayListP a, int e)) {
        return size(a)+1;
    }

    rewrite Object get(Object ArrayListP(), int i) {
        return -1;
    }

    rewrite Object get(Object pushback!(ArrayListP a, int e1), int i) {
        return size(a) == i ? e1 : get(a, i);
    }
}


@JavaCodeGen
public class Test {
    public static void test_arraylist(ArrayListP list, int p, int s, int offset) {
        int adder = ??;
        int factor = ??;
        for (int i = 0; i < p; i++) {
            list.pushback(i * (s % factor + adder) + offset);
        }
    }
}

public class Main {
    harness public static void main(int p, int s, int offset) {
        assume p <= 3;
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