public class ArrayListP {
    int[] storage;
    int size;
    public ArrayListP() {
        storage = new int[20];
        size = 0;
    }
    public void push_back(int input) {
        if (size == 20) {
            return;
        }
        storage[size] = input;
        size = size + 1;
        return;
    }
    public int get(int index){
        if (index >= size) {
            return -1;
        }
        return storage[index];
    }
}


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