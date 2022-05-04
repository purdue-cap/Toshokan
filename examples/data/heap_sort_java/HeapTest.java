public class HeapTest {
    public static void main(String[] args) {
        Heap h = new Heap();
        h.insert(20);
        h.insert(10);
        h.insert(82);
        h.insert(8);
        for (int i = 0; i < 4; i ++) {
            System.out.println(h.popMin());
        }
    }
}