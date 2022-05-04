public class Main {
    public static void main(int p_0, int p_1, int p_2, int p_3, int x) {
        int[] p = new int[]{p_0, p_1, p_2, p_3};

        int num = EvalPoly.evalpoly(p, x);

        int ref_num = 0;
        for (int j=0; j < p.length; j++){
            ref_num += p[j] * Library.pow(x, j);
        }

        assert num == ref_num;

    }
}

