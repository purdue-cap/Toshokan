@rewriteClass
class ArrayListP{
    @alg
    Object add(Integer e);

    @alg
    @pure
    Integer get(int i);

    @alg
    Integer set(int i, Integer e);

    @alg
    @pure
    int size();

    rewrite int size(ArrayListP ArrayListP()) {
	return 0;
    }

    rewrite int size(ArrayListP add!(ArrayListP a, Integer e)) {
	return size(a)+1;
    }

    rewrite int size(ArrayListP set!(ArrayListP a, int i, Integer e)) {
	return size(a);
    }

    rewrite Integer get(ArrayListP add!(ArrayListP a, Integer e1),
		  int i) {
	return size(a) == i ? e1 : get(a, i);
    }

    rewrite Integer get(ArrayListP set!(ArrayListP a, int j, Integer e),
		  int i) {
	return i==j ? e : get(a, i);
    }
}



@JavaCodeGen
public class BubbleSort {
    static public int length;
    
    static int init_j(int i) {
        int offset = ??;
        int choice = ??;
        if (choice == 0) {
            return offset;
        } else if (choice == 1) {
            return i + offset;
        } else if (choice == 2) {
            return i - offset;
        }
        assert false;
        return 0;
    }

    static int target(int j) {
        int offset = ??;
        int choice = ??;
        if (choice == 0) {
            return j + offset;
        } else if (choice == 1) {
            return j - offset;
        }
        assert false;
        return 0;
    }

    static boolean compare(int l, int r) {
        int choice = ??;
        if (choice == 0) {
            return l > r;
        } else if (choice == 1) {
            return l < r;
        } else if (choice == 2) {
            return l == r;
        } else if (choice == 3) {
            return l != r;
        }
        assert false;
        return false;
    }

    static public void bubbleSort(ArrayListP al) {
        for (int i = 0; i < length - 1; i ++)  {
            // Should be j = 1; j < length - i
            for (int j = init_j(i); j < length - i + ??; j++) {
                // Should be j - 1
                int swap_target = target(j);
                Integer left = al.get(j);
                Integer right = al.get(swap_target);
                // Should be < 
                boolean cond = compare(left.intValue(), right.intValue());
                if (cond) {
                    al.set(j, right);
                    al.set(swap_target, left);
                }
            }
        }
    }
}

public class Main {
    harness public static void main(int x, int y, int z) {
    	assume x != y && x != z && y != z;
    	assume x > 0 && x < 10;
    	assume y > 0 && y < 10;
    	assume z > 0 && z < 10;

        ArrayListP al = new ArrayListP();
        Integer xx = new Integer(x);
        Integer yy = new Integer(y);
        Integer zz = new Integer(z);
        al.add(xx);
        al.add(yy);
        al.add(zz);

        BubbleSort.length = al.size();
        BubbleSort.bubbleSort(al);

        for (int i = 0; i < BubbleSort.length - 1; i ++) {
            Integer left = al.get(i);
            Integer right = al.get(i+1);
            assert left.intValue() <= right.intValue();
        }
    }
}

