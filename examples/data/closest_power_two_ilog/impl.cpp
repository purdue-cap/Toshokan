
int pow(int a, int b){
    int result = 1;
    for(int i=0; i<b; i++)
       result = result * a;
    return result;
}

int log_impl(int a){
 if(a == 0) return 0;
 for(int i=0; i<a; i++){
  if(pow(2,i)>=a) return i;
 }
 return 0;
}
