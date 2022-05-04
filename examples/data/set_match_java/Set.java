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

