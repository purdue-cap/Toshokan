#include<cstdlib>
namespace ANONYMOUS{
class Set; 
class Set{
  public:
  int  size;
  int  _hist_len;
  int*  storage;
  int  _hist[];
  Set(){}
template<typename T_0, typename T_1>
  static Set* create(  T_0* storage_, int storage_len,   int  size_,   T_1* _hist_, int _hist_len,   int  _hist_len_);
  ~Set(){
  }
  void operator delete(void* p){ free(p); }
};
};

using ANONYMOUS::Set;

int ANONYMOUS__set_add_real_impl(Set* s, int input) {
  int _out = 0;
  bool  __sa0=(0) < (s->size);
  int  i=0;
  while (__sa0) {
    if (((s->storage[i])) == (input)) {
      _out = 2;
      return _out;
    }
    i = i + 1;
    __sa0 = (i) < (s->size);
  }
  if ((s->size) == (20)) {
    _out = 0;
    return _out;
  }
  (s->storage[s->size]) = input;
  s->size = s->size + 1;
  _out = 1;
  return _out;
}
int ANONYMOUS__set_contains_real_impl(Set* s, int input) {
  int _out = 0;
  bool  __sa1=(0) < (s->size);
  int  i=0;
  while (__sa1) {
    if (((s->storage[i])) == (input)) {
      _out = 1;
      return _out;
    }
    i = i + 1;
    __sa1 = (i) < (s->size);
  }
  _out = 0;
  return _out;
}