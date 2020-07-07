#include <vector>
#include <algorithm>

std::vector<int> ANONYMOUS__sort_proxy_real_impl(int input_0, int input_1, int input_2, int input_3, int input_4) {
   std::vector<int> output = {input_0,input_1,input_2,input_3,input_4};
   int k=0;
   for(int i=0; i<5; ++i){
      for(int j=i+1; j<5; ++j){
         if( output[j]< output[i]){
            int tmp = output[j];
            output[j] = output[i];
            output[i] = tmp;
         }
      }
   }
   return output;
}