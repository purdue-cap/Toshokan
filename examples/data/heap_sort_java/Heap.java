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
