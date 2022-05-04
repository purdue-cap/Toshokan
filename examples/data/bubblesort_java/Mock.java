public class ArrayListP {
    int[] storage;
    int len;
    public ArrayListP() {
        storage = new int[20];
        len = 0;
    }
    public int size() {
        return len;
    }
    public void set(int index, int input) {
        if (index >= len) {
            return;
        }
        storage[index] = input;
    }
    public void add(int input) {
        if (len == 20) {
            return;
        }
        storage[len] = input;
        len = len + 1;
        return;
    }
    public int get(int index){
        if (index >= len) {
            return -1;
        }
        return storage[index];
    }
}



@JavaCodeGen
public class BubbleSort {
    static public int length;
    
    static int init_j(int i) {
        int offset = ??;
        int choice = ??;
        if (choice == 0) {
            return offset;
        } else if (choice == 1) {
            return i + offset;
        } else if (choice == 2) {
            return i - offset;
        }
        assert false;
        return 0;
    }

    static int target(int j) {
        int offset = ??;
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
            for (int j = init_j(i); j < length - i + ??; j++) {
                // Should be j - 1
                int swap_target = target(j);
                int left = al.get(j);
                int right = al.get(swap_target);
                // Should be < 
                boolean cond = compare(left, right);
                if (cond) {
                    al.set(j, right);
                    al.set(swap_target, left);
                }
            }
        }
    }
}

public class Main {
    harness public static void main(int x, int y, int z) {
    	assume x != y && x != z && y != z;
    	assume x > 0 && x < 10;
    	assume y > 0 && y < 10;
    	assume z > 0 && z < 10;

        ArrayListP al = new ArrayListP();
        al.add(x);
        al.add(y);
        al.add(z);

        BubbleSort.length = al.size();
        BubbleSort.bubbleSort(al);

        for (int i = 0; i < BubbleSort.length - 1; i ++) {
            int left = al.get(i);
            int right = al.get(i+1);
            assert left <= right;
        }
    }
}

