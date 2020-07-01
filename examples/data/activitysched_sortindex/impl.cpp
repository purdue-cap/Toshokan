#include <vector>
constexpr int N = 3;

std::vector<int> ANONYMOUS__sorti_proxy_impl(int input_0, int input_1, int input_2) {
   int input[N] = {input_0, input_1, input_2};
   int ind[N] = {0,0,0};
   for(int i=0; i<N; ++i) ind[i] = i;
   int done[N] = {0,0,0};
   int k=0;
   for(int i=0; i<N; ++i){
      for(int j=i+1; j<N; ++j){
         if( input[ind[j]]< input[ind[i]]){
            int tmp2 = ind[j];
            ind[j] = ind[i];
            ind[i] = tmp2;
         }
      }
   }

   return std::vector<int>(ind, ind+3);
}