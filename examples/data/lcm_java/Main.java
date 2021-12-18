public class Main {
    public static void main(int x0, int x1, int x2, int x3, int x4) {
        // 3 , 4, 5 12
        int [] array = new int[] {x0, x1, x2, x3, x4};
        for (int i = 0; i < array.length; i++)
            if (array[i] == 0) return;
        if (array.length < 2) return;
        int result = LCM.lcm_n(array);

        for (int i = 0; i < array.length; i ++)
            assert result % array[i] == 0;
        
        for (int i = 1; i < result; i++) {
            boolean divisible = true;
            for (int j = 0; j < array.length; j ++)
                divisible = divisible && (i % array[j] == 0);
            assert !divisible;
        }
    }
}

