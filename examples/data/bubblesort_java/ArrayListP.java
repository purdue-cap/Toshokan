public class ArrayListP{

    int[] elementData;

    private int DEFAULT_CAPACITY;
    private int capacity;
    private int size;
    private static int[] EMPTY_ELEMENTDATA = {};
    private static final int MAX_ARRAY_SIZE = 1000000; // other value causing weird problem in Sketch
    // private static final int MAX_ARRAY_SIZE = 0x7fffffff - 8;

    public ArrayListP() {
	this.DEFAULT_CAPACITY = 10;
	this.elementData = new int[this.DEFAULT_CAPACITY];
	this.capacity = this.DEFAULT_CAPACITY;
	this.size = 0;
    }

    // Expand capacity to size while keeping old elements of elementData
    private void copyNewElementData(int size) {
	int[] newElementData = new int[size];
	int i = 0;

	for (i = 0; i < this.size; i++) {
	    newElementData[i] = elementData[i];
	}

	elementData = newElementData;
	capacity = size;
    }

    // if adding one would be out of bounds, expand elementData
    private void checkAdjustSize() {
	if (size + 1 >= capacity) {
	    // Arbitrarily 10, should compare to source
	    copyNewElementData(capacity + 10);
	}
    }

    private void createSpace(int index) {
	int j = 0;

	// Note - 1 because one after last element could be out of range
	for (j = size; j > index; j--) {
	    elementData[j] = elementData[j-1];
	}
    }

    public void add(int e) {
	checkAdjustSize();
	elementData[size++] = e;
    return;
    }

    public int get(int index) {
	if (index < 0 || index >= size) {
	    return -1;
	}

	return elementData[index];
    }

    public void set (int index, int element) {

	if (index < 0 || index >= size) {
	    return;
	}

	elementData[index] = element;

	return; 
    }
    
    public int size() {
	return size;
    }

    public void ensureCapacity(int minCapacity) {
	int minExpand;
	if (elementData != EMPTY_ELEMENTDATA) { minExpand = 0; }
	else { minExpand = DEFAULT_CAPACITY; }
	if (minCapacity > minExpand) { ensureExplicitCapacity(minCapacity); }
    }

    private void ensureCapacityInternal(int minCapacity) {
        if (elementData == EMPTY_ELEMENTDATA) {
	    if (DEFAULT_CAPACITY > minCapacity) { minCapacity = DEFAULT_CAPACITY; }
        }
        ensureExplicitCapacity(minCapacity);
    }

    private void ensureExplicitCapacity(int minCapacity) {
        // modCount++; // What is this?

        // overflow-conscious code
        if (minCapacity - elementData.length > 0)
            grow(minCapacity);
    }

    private void grow(int minCapacity) {
        // overflow-conscious code
        int oldCapacity = elementData.length;
        int newCapacity = oldCapacity + (oldCapacity / 2);
        if (newCapacity - minCapacity < 0)
            newCapacity = minCapacity;
        if (newCapacity - MAX_ARRAY_SIZE > 0)
            newCapacity = hugeCapacity(minCapacity);
        // minCapacity is usually close to size, so this is a win:
	copyNewElementData(newCapacity);
    }

    private static int hugeCapacity(int minCapacity) {
        return (minCapacity > MAX_ARRAY_SIZE) ?
            0x7fffffff :
            MAX_ARRAY_SIZE;
    }
}

