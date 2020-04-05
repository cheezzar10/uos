pub fn cmp(s1: &str, s2: &str) -> i32 {
	let s1b = s1.as_bytes();
	let s2b = s2.as_bytes();

	if s1b.len() < s2b.len() {
		return -1
	} else if s1b.len() > s2b.len() {
		return 1
	}

	let s_len = s1b.len();

	let mut i: usize = 0;
	while s1b[i] == s2b[i] {
		if i + 1 == s_len {
			break;
		}

		i += 1;
	}

	if s1b[i] == s2b[i] {
		0
	} else if s1b[i] < s2b[i] {
		-1
	} else {
		1
	}
}

