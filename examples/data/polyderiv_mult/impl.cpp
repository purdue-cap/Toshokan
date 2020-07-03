#include<vector>

constexpr int n = 20;
constexpr int BASE = 4;

std::vector<int> ANONYMOUS__mult_proxy_impl(
    int x_0,int x_1,int x_2,int x_3,int x_4,int x_5,int x_6,int x_7,int x_8,int x_9,int x_10,int x_11,int x_12,int x_13,int x_14,int x_15,int x_16,int x_17,int x_18,int x_19,
    int y_0,int y_1,int y_2,int y_3,int y_4,int y_5,int y_6,int y_7,int y_8,int y_9,int y_10,int y_11,int y_12,int y_13,int y_14,int y_15,int y_16,int y_17,int y_18,int y_19
) {
    std::vector<int> x = {x_0,x_1,x_2,x_3,x_4,x_5,x_6,x_7,x_8,x_9,x_10,x_11,x_12,x_13,x_14,x_15,x_16,x_17,x_18,x_19};
    std::vector<int> y = {y_0,y_1,y_2,y_3,y_4,y_5,y_6,y_7,y_8,y_9,y_10,y_11,y_12,y_13,y_14,y_15,y_16,y_17,y_18,y_19};
    std::vector<int> out = {0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0};
    for(int i=0; i<n; ++i){
        for(int j=0; j<n; ++j){            
            int tmp = y[i] * x[j];
            tmp = out[j + i] + tmp;
            out[j + i] = tmp % BASE;
            out[j + i + 1] = out[j + i + 1] + (tmp / BASE); 
        }           
    }       
    return out;	
}
