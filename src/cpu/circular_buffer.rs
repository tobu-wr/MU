pub(super) struct CircularBuffer<T> {
	elements: Vec::<T>,
	start_index: usize
}

impl<T> CircularBuffer<T> {
	pub(super) fn new(capacity: usize) -> Self {
		Self {
			elements: Vec::<T>::with_capacity(capacity),
			start_index: 0
		}
	}

	pub(super) fn push(&mut self, value: T) {
		if self.elements.len() < self.elements.capacity() {
			self.elements.push(value);
		} else {
			self.elements[self.start_index] = value;
			self.increment_index();
		}
	}

	fn increment_index(&mut self) {
		if self.start_index == (self.elements.capacity() - 1) {
			self.start_index = 0;
		} else {
			self.start_index += 1;
		}
	}
}

impl<T> std::iter::Iterator for CircularBuffer<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		
	}
}
