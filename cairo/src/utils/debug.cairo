from debug import print_felt_hex, print_string

func print_segment_hex(segment_ptr: felt*, len: felt, index: felt) {
    if (index == len) {
        return ();
    }
    print_string(index);
    print_felt_hex([segment_ptr + index]);
    return print_segment_hex(segment_ptr=segment_ptr, len=len, index=index + 1);
}
