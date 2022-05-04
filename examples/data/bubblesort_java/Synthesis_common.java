@JavaCodeGen
public class BubbleSort {
    static public int length;
    
    // static int init_j(int i) {
    //     int offset = ??;
    //     int choice = ??;
    //     if (choice == 0) {
    //         return offset;
    //     } else if (choice == 1) {
    //         return i + offset;
    //     } else if (choice == 2) {
    //         return i - offset;
    //     }
    //     assert false;
    //     return 0;
    // }

    static int target(int j) {
        int offset = 1;
        int choice = ??;
        if (choice == 0) {
            return j + offset;
        } else if (choice == 1) {
            return j - offset;
        }
        assert false;
        return 0;
    }

    static boolean compare(int l, int r) {
        int choice = ??;
        if (choice == 0) {
            return l > r;
        } else if (choice == 1) {
            return l < r;
        } else if (choice == 2) {
            return l == r;
        } else if (choice == 3) {
            return l != r;
        }
        assert false;
        return false;
    }

    static public void bubbleSort(ArrayListP al) {
        for (int i = 0; i < length - 1; i ++)  {
            // Should be j = 1; j < length - i
            // for (int j = init_j(i); j < length - i + ??; j++) {
            for (int j = 1; j < length - i; j++) {
                // Should be j - 1
                int swap_target = target(j);
                // int swap_target = j - 1;
                int left = al.get(j);
                int right = al.get(swap_target);
                // Should be < 
                boolean cond = compare(left, right);
                // boolean cond = left < right;
                if (cond) {
                    al.set(j, right);
                    al.set(swap_target, left);
                }
            }
        }
    }
}
