import java.lang.Math;

public class Library {
   public static int sqrt(int num) {
        if (num==1 || num==0) return num;
        if (num<0) assert false;
        int low=0;
        int mid;
        int high=1+(num/2);
        while (low+1<high){
            mid=low+(high-low)/2;
            if (num %mid == 0 && num/mid == mid)
                return mid;
            else if (mid<=num/mid)
                low=mid;
            else
                high=mid;
        }
        return low;
   }
}
