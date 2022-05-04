public class Stack {
    int[] storage;
    int pos;
    public Stack() {
        storage = new int[20];
        pos = 0;
    }
    public void push(int i) {
        if (pos >= 20) {
            return;
        }
        storage[pos] = i;
        pos = pos + 1;
    }
    public int pop() {
        if (pos == 0) {
            return 0;
        }
        pos = pos - 1;
        return storage[pos];
    }
}
