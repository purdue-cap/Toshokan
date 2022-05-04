@JavaCodeGen
public class HeapSort {
    static boolean cond(int i, int N) {
        int choice = ??;
        int expr_choice = ??;
        // int choice = 0;
        // int expr_choice = 0;
        int expr = 0;
        if (expr_choice == 0) {
            expr = N;
        } else if (expr_choice == 1) {
            expr = N + 1;
        } else if (expr_choice == 2) {
            expr = N - 1;
        } else {
            assert false;
        }
        if (choice == 0) {
            return i < expr;
        } else if (choice == 1) {
            return i <= expr;
        } else if (choice == 2) {
            return i > expr;
        } else if (choice == 3) {
            return i >= expr;
        }
        assert false;
        return false;
    }
    static int incr(int i) {
        int choice = ??;
        // int choice = 0;
        if (choice == 0) {
            return i + 1;
        } else if (choice == 1) {
            return i + 2;
        } else if (choice == 2) {
            return i - 1;
        } else if (choice == 3) {
            return i - 2;
        }
        assert false;
        return 0;

    }
    static int index(int i) {
        // int choice = 0;
        int choice = ??;
        if (choice == 0) {
            return i;
        } else if (choice == 1) {
            return i + 1;
        } else if (choice == 2) {
            return i - 1;
        }
        assert false;
        return 0;
    }
    public static int[] heapsort(int[] input) {
        Heap h = new Heap();
        for (int i = 0; i < input.length; i++) {
            h.insert(input[i]);
        }
        // int start = 0;
        int start = ??;
        int[] output = new int[input.length];
        for (int i = start; cond(i, input.length); i = incr(i)) {
            output[index(i)] = h.popMin();
        }
        return output;
    }
}