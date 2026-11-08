// the jdk 6 Arrays.sort(Object[], Comparator) legacy merge sort that Collections.sort also uses

const INSERTIONSORT_THRESHOLD: i32 = 7;

pub fn sort<T: Clone>(items: &mut [T], compare: &mut impl FnMut(&T, &T) -> i32)
{
    let mut aux: Vec<T> = items.to_vec();
    let length: i32 = items.len() as i32;
    mergeSort(&mut aux, items, 0, length, 0, compare);
}

fn mergeSort<T: Clone>(
    src: &mut [T],
    dest: &mut [T],
    low: i32,
    high: i32,
    off: i32,
    compare: &mut impl FnMut(&T, &T) -> i32,
)
{
    let length: i32 = high - low;
    if length < INSERTIONSORT_THRESHOLD
    {
        for i in low..high
        {
            let mut j: i32 = i;
            while j > low && compare(&dest[(j - 1) as usize], &dest[j as usize]) > 0
            {
                dest.swap(j as usize, (j - 1) as usize);
                j -= 1;
            }
        }
        return;
    }

    let destLow: i32 = low;
    let destHigh: i32 = high;
    let low: i32 = low + off;
    let high: i32 = high + off;
    let mid: i32 = ((low + high) as u32 >> 1) as i32;
    mergeSort(dest, src, low, mid, -off, compare);
    mergeSort(dest, src, mid, high, -off, compare);

    if compare(&src[(mid - 1) as usize], &src[mid as usize]) <= 0
    {
        dest[destLow as usize..(destLow + length) as usize]
            .clone_from_slice(&src[low as usize..(low + length) as usize]);
        return;
    }

    let mut p: i32 = low;
    let mut q: i32 = mid;
    for i in destLow..destHigh
    {
        if q >= high || (p < mid && compare(&src[p as usize], &src[q as usize]) <= 0)
        {
            dest[i as usize] = src[p as usize].clone();
            p += 1;
        }
        else
        {
            dest[i as usize] = src[q as usize].clone();
            q += 1;
        }
    }
}
