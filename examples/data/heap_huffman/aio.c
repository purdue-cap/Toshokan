#include <stdlib.h>
#include <stdio.h>
#define HEAP_SIZE 20

typedef struct {
    int count;
    int arr[HEAP_SIZE];
    int value[HEAP_SIZE];
} Heap;

typedef struct {
    int key;
    int value; // if < 100, char values, if >= 100, index in internal node array
} Entry;

typedef struct Node{
    int key;
    int value;
    struct Node* left;
    struct Node* right;
} Node;

Heap* heap_new(){
    Heap* h = (Heap*)malloc(sizeof(Heap));
    h->count = 0;
    return h;
}

void heap_heapify_bottom_top(Heap* h,int index);

int heap_insert(Heap* h, int key, int value){
    if (h->count < HEAP_SIZE) {
        h->arr[h->count] = key;
        h->value[h->count] = value;
        heap_heapify_bottom_top(h, h->count);
        h->count++;
        return 1;
    }
    return 0;
}

void heap_heapify_bottom_top(Heap* h,int index){
    int temp;
    int parent_node = (index-1)/2;

    if(h->arr[parent_node] > h->arr[index]){
        //swap and recursive call
        temp = h->arr[parent_node];
        h->arr[parent_node] = h->arr[index];
        h->arr[index] = temp;
        temp = h->value[parent_node];
        h->value[parent_node] = h->value[index];
        h->value[index] = temp;
        heap_heapify_bottom_top(h,parent_node);
    }
}

void heap_heapify_top_bottom(Heap* h, int parent_node){
    int left = parent_node*2+1;
    int right = parent_node*2+2;
    int min;
    int temp;

    if(left >= h->count || left <0)
        left = -1;
    if(right >= h->count || right <0)
        right = -1;

    if(left != -1 && h->arr[left] < h->arr[parent_node])
        min=left;
    else
        min =parent_node;
    if(right != -1 && h->arr[right] < h->arr[min])
        min = right;

    if(min != parent_node){
        temp = h->arr[min];
        h->arr[min] = h->arr[parent_node];
        h->arr[parent_node] = temp;
        temp = h->value[min];
        h->value[min] = h->value[parent_node];
        h->value[parent_node] = temp;

        // recursive  call
        heap_heapify_top_bottom(h, min);
    }
}

Entry heap_pop_min(Heap* h){
    Entry et;
    if(h->count==0){
        return et;
    }
    // replace first node by last and delete last
    et.key = h->arr[0];
    et.value = h->value[0];
    h->arr[0] = h->arr[h->count-1];
    h->value[0] = h->value[h->count-1];
    h->count--;
    heap_heapify_top_bottom(h, 0);
    return et;
}

int heap_size(Heap* h) {
    return h->count;
}

Node* build_huffman_tree(Heap* h) {
    Node* nodes[HEAP_SIZE];
    int node_count = 0;
    Node* root = NULL;
    while(heap_size(h)>1) {
        Entry et_l = heap_pop_min(h);
        Node* left;
        if (et_l.value < 100) {
            left = (Node*)malloc(sizeof(Node));
            left->key = et_l.key;
            left->value = et_l.value;
            left->left = NULL;
            left->right = NULL;
        } else {
            left = nodes[et_l.value%100];
        }
        Entry et_r = heap_pop_min(h);
        Node* right;
        if (et_r.value < 100) {
            right = (Node*)malloc(sizeof(Node));
            right->key = et_r.key;
            right->value = et_r.value;
            right->left = NULL;
            right->right = NULL;
        } else {
            right = nodes[et_r.value%100];
        }
        Node* parent = (Node*)malloc(sizeof(Node));
        parent->key = et_l.key + et_r.key;
        parent->left = left;
        parent->right = right;
        parent->value = 100 + node_count;
        nodes[node_count] = parent;
        node_count++;
        heap_insert(h, parent->key, parent->value);
    }
    return nodes[heap_pop_min(h).value%100];
}

void get_huffman_code(Node* node, int current, int* result) {
    if (node->value < 100) {
        result[node->value] = current;
        return;
    }
    get_huffman_code(node->left, current*2, result);
    get_huffman_code(node->right, current*2+1, result);
}

int main() {
    Heap* h = heap_new();
    int result[4];
    heap_insert(h, 23, 0);
    heap_insert(h, 10, 1);
    heap_insert(h, 40, 2);
    heap_insert(h, 30, 3);
    Node* huffman_tree = build_huffman_tree(h);
    get_huffman_code(huffman_tree, 0, result);
    for (int i=0; i<4; i++) {
        printf("%d %d\n", i, result[i]);
    }
    return 0;
}