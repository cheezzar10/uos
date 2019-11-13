typedef __SIZE_TYPE__ size_t;

void* memset(void* dst, int c, size_t n) {
	char* d = dst;
	for (size_t i=0;i<n;i++) {
		d[i] = c;
	}
	return dst;
}

void* memcpy(void* dst, const void* src, size_t n) {
	char* d = dst;
	const char* s = src;

	for (size_t i = 0;i < n;i++) {
		d[i] = s[i];
	}

	return dst;
}

int memcmp(const void* s1, const void* s2, size_t n) {
	const char* l = s1;
	const char* r = s2;

	size_t i = 0;
	for (;i < n && l[i] == r[i];i++);

	if (l[i] == r[i]) {
		return 0;
	} else if (l[i] < r[i]) {
		return -1;
	} else {
		return 1;
	}
}

int bcmp(const void* s1, const void* s2, size_t n) {
	return memcmp(s1, s2, n);
}
