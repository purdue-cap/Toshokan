#include<cstdlib>
namespace ANONYMOUS{
class ArrayList; 
class ArrayList{
  public:
  int  size;
  int  _hist_len;
  int*  storage;
  int  _hist[];
  ArrayList(){}
template<typename T_0, typename T_1>
  static ArrayList* create(  T_0* storage_, int storage_len,   int  size_,   T_1* _hist_, int _hist_len,   int  _hist_len_);
  ~ArrayList(){
  }
  void operator delete(void* p){ free(p); }
};
};

using ANONYMOUS::ArrayList;

int ANONYMOUS__arraylist_push_back_real_impl(ArrayList* l, int input) {
  int _out = 0;
  if ((l->size) == (20)) {
    _out = 0;
    return _out;
  }
  (l->storage[l->size]) = input;
  l->size = l->size + 1;
  _out = 1;
  return _out;
}
int ANONYMOUS__arraylist_get_real_impl(ArrayList* l, int index) {
  int _out = 0;
  if ((index) >= (l->size)) {
    _out = -1;
    return _out;
  }
  _out = (l->storage[index]);
  return _out;
}