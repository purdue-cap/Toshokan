// import org.cprover.CProver;
public class Main {
    public static void main(int i0, int i1, int i2) {
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

