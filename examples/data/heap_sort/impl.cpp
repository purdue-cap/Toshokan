#include<cstdlib>
namespace ANONYMOUS{
class Heap; 
class Heap{
  public:
  int  count;
  int  _hist_len;
  int*  arr;
  int  _hist[];
  Heap(){}
template<typename T_0, typename T_1>
  static Heap* create(  int  count_,   T_0* arr_, int arr_len,   T_1* _hist_, int _hist_len,   int  _hist_len_);
  ~Heap(){
  }
  void operator delete(void* p){ free(p); }
};
};

void heap_heapify_bottom_top(ANONYMOUS::Heap* h, int index) {
  int  parent_node=(index - 1) / 2;
  int  temp=0;
  if (((h->arr[parent_node])) > ((h->arr[index]))) {
    temp = (h->arr[parent_node]);
    (h->arr[parent_node]) = (h->arr[index]);
    (h->arr[index]) = temp;
    heap_heapify_bottom_top(h, parent_node);
  }
}
void heap_heapify_top_bottom(ANONYMOUS::Heap* h, int parent_node) {
  int  left=(parent_node * 2) + 1;
  int  right=(parent_node * 2) + 2;
  if (((left) >= (h->count)) || ((left) < (0))) {
    left = -1;
  }
  if (((right) >= (h->count)) || ((right) < (0))) {
    right = -1;
  }
  int  min=0;
  if (((left) != (-1)) && (((h->arr[left])) < ((h->arr[parent_node])))) {
    min = left;
  } else {
    min = parent_node;
  }
  if (((right) != (-1)) && (((h->arr[right])) < ((h->arr[min])))) {
    min = right;
  }
  int  temp=0;
  if ((min) != (parent_node)) {
    temp = (h->arr[min]);
    (h->arr[min]) = (h->arr[parent_node]);
    (h->arr[parent_node]) = temp;
    heap_heapify_top_bottom(h, min);
  }
}

int ANONYMOUS__heap_insert_real_impl(ANONYMOUS::Heap* h, int key) {
  int _out;
  if ((h->count) < (20)) {
    (h->arr[h->count]) = key;
    heap_heapify_bottom_top(h, h->count);
    h->count = h->count + 1;
    _out = 1;
    return _out;
  }
  _out = 0;
  return _out;
}

int ANONYMOUS__heap_pop_min_real_impl(ANONYMOUS::Heap* h) {
  int _out;
  if ((h->count) == (0)) {
    _out = -1;
    return _out;
  }
  int  pop=(h->arr[0]);
  (h->arr[0]) = (h->arr[h->count - 1]);
  h->count = h->count - 1;
  heap_heapify_top_bottom(h, 0);
  _out = pop;
  return _out;
}
