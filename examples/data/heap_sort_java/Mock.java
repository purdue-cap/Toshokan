public class Heap {
    static final int HEAP_SIZE = 20;
    int count;
    int[] arr;

    public Heap() {
        count = 0;
        arr = new int[HEAP_SIZE];
    }

    public void insert(int key) {
        if (count < HEAP_SIZE) {
            arr[count] = key;
            heap_heapify_bottom_top(count);
            count++;
        }
    }
    void heap_heapify_bottom_top(int index){
        int temp;
        int parent_node = (index-1)/2;

        if(arr[parent_node] > arr[index]){
            //swap and recursive call
            temp = arr[parent_node];
            arr[parent_node] = arr[index];
            arr[index] = temp;
            heap_heapify_bottom_top(parent_node);
        }
    }

    void heap_heapify_top_bottom(int parent_node){
        int left = parent_node*2+1;
        int right = parent_node*2+2;
        int min;
        int temp;

        if(left >= count || left <0)
            left = -1;
        if(right >= count || right <0)
            right = -1;

        if(left != -1 && arr[left] < arr[parent_node])
            min=left;
        else
            min =parent_node;
        if(right != -1 && arr[right] < arr[min])
            min = right;

        if(min != parent_node){
            temp = arr[min];
            arr[min] = arr[parent_node];
            arr[parent_node] = temp;

            // recursive  call
            heap_heapify_top_bottom(min);
        }
    }

    public int popMin(){
        int pop;
        if(count==0){
            return -1;
        }
        // replace first node by last and delete last
        pop = arr[0];
        arr[0] = arr[count-1];
        count--;
        heap_heapify_top_bottom(0);
        return pop;
    }
}

@JavaCodeGen
public class HeapSort {
    static boolean cond(int i, int N) {
        int choice = ??;
        int expr_choice = ??;
        int expr;
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
        int start = ??;
        int[] output = new int[input.length];
        for (int i = start; cond(i, input.length); i = incr(i)) {
            output[index(i)] = h.popMin();
        }
        return output;
    }
}

public class Main {
    harness public static void main(int i0, int i1, int i2) {
        int[] input = new int[]{i0, i1, i2};
        int[] output = HeapSort.heapsort(input);

        for (int i = 0; i < input.length; i++) {
            boolean present_in_output = false;
            for (int j = 0; j < input.length; j++) {
                if (output[j] == input[i]) present_in_output = true;
            }
            assert present_in_output;
        }
        for (int i = 0; i < input.length-1; i++) {
            assert output[i] <= output[i+1];
        }
    }
}

