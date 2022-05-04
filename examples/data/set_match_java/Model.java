@rewriteClass
public class SetI {
    @alg
    Object addi(int input);
    
    @alg
    @pure
    int size();

    @alg
    @pure
    int contains(int input);

    rewrite Object size(Object SetI()) {
        return 0;
    }

    rewrite Object size(Object addi!(SetI s, int i)) {
        return size(s) + 1;
    }

    rewrite Object contains(Object SetI(), int i) {
        return 0;
    }

    rewrite Object contains(Object addi!(SetI s, int i1), int i2) {
        return i1 == i2 ? 1 : contains(s, i2);
    }
}


@JavaCodeGen
public class Test {
    public static void test_set(SetI st, int p, int s, int offset) {
        int addier = ??;
        int factor = ??;
        for (int i = 0; i < p; i++) {
            st.addi(i * (s % factor + addier) + offset);
        }
    }
}

public class Main {
    harness public static void main(int p, int s, int offset) {
        assume p <= 3;
        SetI st = new SetI();

        Test.test_set(st, p, s, offset);

        int check_value = offset;
        for (int i = 0; i < p; i++) {
            int contains = st.contains(check_value);
            assert contains == 1;
            check_value =  check_value + s % 2 + 2;
        }
    }
}