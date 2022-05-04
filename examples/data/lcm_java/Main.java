import org.cprover.CProver;
public class Main {
    public static void main(int x0, int x1, int x2, int x3, int x4) {
        // 3 , 4, 5 12
        CProver.assume(x0 >= 0 && x0 < 5);
        CProver.assume(x1 >= 0 && x1 < 5);
        CProver.assume(x2 >= 0 && x2 < 5);
        CProver.assume(x3 >= 0 && x3 < 5);
        CProver.assume(x4 >= 0 && x4 < 5);
        int [] array = new int[] {x0, x1, x2, x3, x4};
        for (int i = 0; i < array.length; i++)
            if (array[i] == 0) return;
        if (array.length < 2) return;
        int result = LCM.lcm_n(array);

        int k = CProver.nondetInt();
        CProver.assume(k >= 0 && k < array.length);
        assert result % array[k] == 0;
        
        int l = CProver.nondetInt();
        CProver.assume(l >= 1 && l < result);
        boolean divisible = true;
        for (int j = 0; j < array.length; j ++)
            divisible = divisible && (l % array[j] == 0);
        assert !divisible;
    }
}

