public class Set {
    int[] storage;
    int set_size;
    public Set() {
        storage = new int[20];
        set_size = 0;
    }
    public int size() {
        return set_size;
    }
    public int contains(int input) {
        for (int i = 0; i < set_size; i++ ) {
            if (storage[i] == input) {
                return 1;
            }
        }
        return 0;
    }
    public void add(int input){
        for (int i = 0; i < set_size; i++ ) {
            if (storage[i] == input) {
                return;
            }
        }
        if (set_size == 20) {
            return;
        }
        storage[set_size] = input;
        set_size = set_size + 1;
        return;
    }
}


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

public class Main {
    harness public static void main(int p, int s, int offset) {
        assume p <= 3;
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