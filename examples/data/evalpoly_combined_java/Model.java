public class Library {
// Mock version
//    public static int pow(int a, int b) {
//         if (a > 1 && b < 0) {
//             return 0;
//         }
//         int result = 1;
//         for(int i=0; i<b; i++)
//         result = result * a;
//         return result;
//    }
   public static int powuf(int a, int b);

   public static model int pow(int a, int b) {
       int rv = powuf(a, b);

       if (a == 0) {
           if (b > 0) {
                assert rv == 0;
           } else {
               assert rv == 1;
           }
           return rv;
       }

       if (a > 1 && b < 0) {
           assert rv == 0;
           return rv;
       }
       
       int n = rv;
       for (int i = 0; i < b - 1; i++) {
           assert n % a == 0;
           n = n / a;
       }
       assert n == 1;
       return rv;

   }
}

@JavaCodeGen
public class EvalPoly {
    static boolean cond_atom(int i, int length, int choice, int bound) {
        if (choice == 0) {
            return i >= bound;
        } else if (choice == 1) {
            return i <= bound;
        } else if (choice == 2) {
            return i > bound;
        } else if (choice == 3) {
            return i < bound;
        } else if (choice == 4) {
            return i >= length;
        } else if (choice == 5) {
            return i <= length;
        } else if (choice == 6) {
            return i > length;
        } else if (choice == 7) {
            return i < length;
        }
        assert false;
        return false;
    }

    static boolean cond(int i, int length) {
        int compound = ??;
        int choice_left = ??;
        int bound_left = ??;
        int choice_right = ??;
        int bound_right = ??;
        if (compound == 0) {
            return cond_atom(i, length, choice_left, bound_left);
        } else if (compound == 1) {
            return cond_atom(i, length, choice_left, bound_left) && cond_atom(i, length, choice_right, bound_right);
        } else if (compound == 2) {
            return cond_atom(i, length, choice_left, bound_left) || cond_atom(i, length, choice_right, bound_right);
        }
        assert false;
        return false;

    }
    static int expr_atom(int length, int choice, int hole) {
        if (choice == 0) {
            return length;
        } else if (choice == 1) {
            return hole;
        }
        assert false;
        return 0;
    }

    static int expr(int length) {
        int choice = ??;
        int choice_left = ??;
        int hole_left = ??;
        int choice_right = ??;
        int hole_right = ??;
        if (choice == 0) {
            return expr_atom(length, choice_left, hole_left);
        } else if (choice == 1) {
            return expr_atom(length, choice_left, hole_left) * expr_atom(length, choice_right, hole_right);
        } else if (choice == 2) {
            return expr_atom(length, choice_left, hole_left) + expr_atom(length, choice_right, hole_right);
        } else if (choice == 3) {
            return expr_atom(length, choice_left, hole_left) - expr_atom(length, choice_right, hole_right);
        }
        assert false;
        return 0;
    }

    public static int evalpoly(int[] p, int x) {
        int num = 0;
        int i = expr(p.length);
        while (cond(i, p.length)) {
            num += p[i] * Library.pow(x, i);
            i = i - 1;
        }
        return num;
    }
}

public class Main {
    harness public static void main(int p_0, int p_1, int p_2, int p_3, int x) {
        int[] p = new int[]{p_0, p_1, p_2, p_3};

        int num = EvalPoly.evalpoly(p, x);

        int ref_num = 0;
        for (int j=0; j < p.length; j++){
            ref_num += p[j] * Library.pow(x, j);
        }

        assert num == ref_num;

    }
}
