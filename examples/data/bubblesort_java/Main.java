import org.cprover.CProver;
public class Main {

    public static void main(int x, int y, int z) {
    	CProver.assume(x != y && x != z && y != z);
    	CProver.assume(x > 0 && x < 10);
    	CProver.assume(y > 0 && y < 10);
    	CProver.assume(z > 0 && z < 10);

        ArrayListP al = new ArrayListP();
        al.add(x);
        al.add(y);
        al.add(z);

        BubbleSort.length = 3;
        BubbleSort.bubbleSort(al);

        for (int i = 0; i < BubbleSort.length - 1; i ++) {
            int left = al.get(i);
            int right = al.get(i+1);
            assert left <= right;
        }
    }
}

