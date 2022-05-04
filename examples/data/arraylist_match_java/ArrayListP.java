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

