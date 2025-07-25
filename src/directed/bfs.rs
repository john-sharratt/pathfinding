//! Compute a shortest path using the [breadth-first search
//! algorithm](https://en.wikipedia.org/wiki/Breadth-first_search).

use super::reverse_path;
use crate::NodeRefs;
use indexmap::map::Entry::Vacant;
use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHasher;
use std::hash::{BuildHasher, BuildHasherDefault, Hash};
use std::iter::FusedIterator;

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search).
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
///   a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
///
/// # Example
///
/// We will search the shortest path on a chess board to go from (1, 1) to (4, 6) doing only knight
/// moves.
///
/// The first version uses an explicit type `Pos` on which the required traits are derived.
///
/// ```
/// use pathfinding::prelude::bfs;
///
/// #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// struct Pos(i32, i32);
///
/// impl Pos {
///   fn successors(&self) -> Vec<Pos> {
///     let &Pos(x, y) = self;
///     vec![Pos(x+1,y+2), Pos(x+1,y-2), Pos(x-1,y+2), Pos(x-1,y-2),
///          Pos(x+2,y+1), Pos(x+2,y-1), Pos(x-2,y+1), Pos(x-2,y-1)]
///   }
/// }
///
/// static GOAL: Pos = Pos(4, 6);
/// let result = bfs(&Pos(1, 1), |p| p.successors(), |p| *p == GOAL);
/// assert_eq!(result.expect("no path found").len(), 5);
/// ```
///
/// The second version does not declare a `Pos` type, makes use of more closures,
/// and is thus shorter.
///
/// ```
/// use pathfinding::prelude::bfs;
///
/// static GOAL: (i32, i32) = (4, 6);
/// let result = bfs(&(1, 1),
///                  |&(x, y)| vec![(x+1,y+2), (x+1,y-2), (x-1,y+2), (x-1,y-2),
///                                 (x+2,y+1), (x+2,y-1), (x-2,y+1), (x-2,y-1)],
///                  |&p| p == GOAL);
/// assert_eq!(result.expect("no path found").len(), 5);
/// ```
pub fn bfs<'a, N, S, FN, IN, FS>(start: S, successors: FN, success: FS) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    S: Into<NodeRefs<'a, N>>,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    bfs_with_hasher(start, successors, success, BuildHasherDefault::<FxHasher>::default())
}

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search) with a custom hasher.
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
///   a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
pub fn bfs_with_hasher<'a, N, S, FN, IN, FS, H>(start: S, successors: FN, success: FS, hasher: H) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    S: Into<NodeRefs<'a, N>>,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
    H: BuildHasher,
{
    bfs_core(&start.into(), successors, success, true, hasher)
}

fn bfs_core<'a, N, FN, IN, FS, H>(
    start: &NodeRefs<'a, N>,
    mut successors: FN,
    mut success: FS,
    check_first: bool,
    hasher: H
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
    H: BuildHasher,
{
    if check_first {
        for start_node in start {
            if success(start_node) {
                return Some(vec![start_node.clone()]);
            }
        }
    }

    let mut parents: IndexMap<N, usize, H> = IndexMap::with_hasher(hasher);
    parents.extend(start.into_iter().map(|n| (n.clone(), usize::MAX)));

    let mut i = 0;
    while let Some((node, _)) = parents.get_index(i) {
        for successor in successors(node) {
            if success(&successor) {
                let mut path = reverse_path(&parents, |&p| p, i);
                path.push(successor);
                return Some(path);
            }
            if let Vacant(e) = parents.entry(successor) {
                e.insert(i);
            }
        }
        i += 1;
    }
    None
}

/// Return one of the shortest loop from start to start if it exists, `None` otherwise.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
///
/// Except the start node which will be included both at the beginning and the end of
/// the path, a node will never be included twice in the path as determined
/// by the `Eq` relationship.
pub fn bfs_loop<'a, N, S, FN, IN>(start: S, successors: FN) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    S: Into<NodeRefs<'a, N>>,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    bfs_loop_with_hasher(start, successors, BuildHasherDefault::<FxHasher>::default())
}

/// Return one of the shortest loop from start to start if it exists, `None` otherwise using
/// a custom hasher.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
///
/// Except the start node which will be included both at the beginning and the end of
/// the path, a node will never be included twice in the path as determined
/// by the `Eq` relationship.
pub fn bfs_loop_with_hasher<'a, N, S, FN, IN, H>(start: S, successors: FN, hasher: H) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    S: Into<NodeRefs<'a, N>>,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    H: BuildHasher,
{
    let start = start.into();
    bfs_core(&start, successors, |n| start.contains(n), false, hasher)
}

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search) with
/// [bidirectional search](https://en.wikipedia.org/wiki/Bidirectional_search).
///
/// Bidirectional search runs two simultaneous searches: one forward from the start,
/// and one backward from the end, stopping when the two meet. In many cases this gives
/// a faster result than searching only in a single direction.
///
/// The shortest path starting from `start` up to a node `end` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `end` is the end node.
/// - `successors_fn` returns a list of successors for a given node.
/// - `predecessors_fn` returns a list of predecessors for a given node. For an undirected graph
///   this will be the same as `successors_fn`, however for a directed graph this will be different.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
///
/// # Example
///
/// We will search the shortest path on a chess board to go from (1, 1) to (4, 6) doing only knight
/// moves.
///
/// ```
/// use pathfinding::prelude::bfs_bidirectional;
///
/// static SUCCESSORS: fn(&(i32, i32)) -> Vec<(i32, i32)> = |&(x, y)| vec![
///     (x+1,y+2), (x+1,y-2), (x-1,y+2), (x-1,y-2),
///     (x+2,y+1), (x+2,y-1), (x-2,y+1), (x-2,y-1)
/// ];
/// let result = bfs_bidirectional(&(1, 1), &(4, 6), SUCCESSORS, SUCCESSORS);
/// assert_eq!(result.expect("no path found").len(), 5);
/// ```
///
/// Find also a more interesting example, comparing regular
/// and bidirectional BFS [here](https://github.com/evenfurther/pathfinding/blob/main/examples/bfs_bidirectional.rs).
#[allow(clippy::missing_panics_doc)]
pub fn bfs_bidirectional<'a, N, S, E, FNS, FNP, IN>(
    start: S,
    end: E,
    successors_fn: FNS,
    predecessors_fn: FNP,
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    E: Into<NodeRefs<'a, N>>,
    S: Into<NodeRefs<'a, N>>,
    FNS: Fn(&N) -> IN,
    FNP: Fn(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    bfs_bidirectional_with_hasher(start, end, successors_fn, predecessors_fn, BuildHasherDefault::<FxHasher>::default())
}

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search) with
/// [bidirectional search](https://en.wikipedia.org/wiki/Bidirectional_search) with a custom hasher.
///
/// Bidirectional search runs two simultaneous searches: one forward from the start,
/// and one backward from the end, stopping when the two meet. In many cases this gives
/// a faster result than searching only in a single direction.
///
/// The shortest path starting from `start` up to a node `end` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `end` is the end node.
/// - `successors_fn` returns a list of successors for a given node.
/// - `predecessors_fn` returns a list of predecessors for a given node. For an undirected graph
///   this will be the same as `successors_fn`, however for a directed graph this will be different.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
#[allow(clippy::missing_panics_doc)]
pub fn bfs_bidirectional_with_hasher<'a, N, S, E, FNS, FNP, IN, H>(
    start: S,
    end: E,
    successors_fn: FNS,
    predecessors_fn: FNP,
    hasher: H
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    E: Into<NodeRefs<'a, N>>,
    S: Into<NodeRefs<'a, N>>,
    FNS: Fn(&N) -> IN,
    FNP: Fn(&N) -> IN,
    IN: IntoIterator<Item = N>,
    H: BuildHasher + Clone
{
    let start = start.into();
    let end = end.into();

    let mut predecessors: IndexMap<N, Option<usize>, H> = IndexMap::with_hasher(hasher.clone());
    predecessors.extend(start.into_iter().cloned().map(|n| (n, None)));
    let mut successors: IndexMap<N, Option<usize>, H> = IndexMap::with_hasher(hasher);
    successors.extend(end.into_iter().cloned().map(|n| (n, None)));

    let mut i_forwards = 0;
    let mut i_backwards = 0;
    let middle = 'l: loop {
        for _ in 0..(predecessors.len() - i_forwards) {
            let node = predecessors.get_index(i_forwards).unwrap().0;
            for successor_node in successors_fn(node) {
                if !predecessors.contains_key(&successor_node) {
                    predecessors.insert(successor_node.clone(), Some(i_forwards));
                }
                if successors.contains_key(&successor_node) {
                    break 'l Some(successor_node);
                }
            }
            i_forwards += 1;
        }

        for _ in 0..(successors.len() - i_backwards) {
            let node = successors.get_index(i_backwards).unwrap().0;
            for predecessor_node in predecessors_fn(node) {
                if !successors.contains_key(&predecessor_node) {
                    successors.insert(predecessor_node.clone(), Some(i_backwards));
                }
                if predecessors.contains_key(&predecessor_node) {
                    break 'l Some(predecessor_node);
                }
            }
            i_backwards += 1;
        }

        if i_forwards == predecessors.len() && i_backwards == successors.len() {
            break 'l None;
        }
    };

    middle.map(|middle| {
        // Path found!
        // Build the path.
        let mut path = vec![];
        // From middle to the start.
        let mut node = Some(middle.clone());
        while let Some(n) = node {
            path.push(n.clone());
            node = predecessors[&n].map(|i| predecessors.get_index(i).unwrap().0.clone());
        }
        // Reverse, to put start at the front.
        path.reverse();
        // And from middle to the end.
        let mut node = successors[&middle].map(|i| successors.get_index(i).unwrap().0.clone());
        while let Some(n) = node {
            path.push(n.clone());
            node = successors[&n].map(|i| successors.get_index(i).unwrap().0.clone());
        }
        path
    })
}

/// Visit all nodes that are reachable from a start node. The node will be visited
/// in BFS order, starting from the `start` node and following the order returned
/// by the `successors` function.
///
/// # Examples
///
/// The iterator stops when there are no new nodes to visit:
///
/// ```
/// use pathfinding::prelude::bfs_reach;
///
/// let all_nodes = bfs_reach(3, |_| (1..=5)).collect::<Vec<_>>();
/// assert_eq!(all_nodes, vec![3, 1, 2, 4, 5]);
/// ```
///
/// The iterator can be used as a generator. Here are for examples
/// the multiples of 2 and 3 (although not in natural order but in
/// the order they are discovered by the BFS algorithm):
///
/// ```
/// use pathfinding::prelude::bfs_reach;
///
/// let mut it = bfs_reach(1, |&n| vec![n*2, n*3]).skip(1);
/// assert_eq!(it.next(), Some(2));  // 1*2
/// assert_eq!(it.next(), Some(3));  // 1*3
/// assert_eq!(it.next(), Some(4));  // (1*2)*2
/// assert_eq!(it.next(), Some(6));  // (1*2)*3
/// // (1*3)*2 == 6 which has been seen already
/// assert_eq!(it.next(), Some(9));  // (1*3)*3
/// assert_eq!(it.next(), Some(8));  // ((1*2)*2)*2
/// assert_eq!(it.next(), Some(12)); // ((1*2)*2)*3
/// ```
pub fn bfs_reach<N, FN, IN>(start: N, successors: FN) -> BfsReachable<N, FN, BuildHasherDefault<FxHasher>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    bfs_reach_with_hasher(start, successors, BuildHasherDefault::<FxHasher>::default())
}

/// Visit all nodes that are reachable from a start node. The node will be visited
/// in BFS order, starting from the `start` node and following the order returned
/// by the `successors` function using a custom hasher.
pub fn bfs_reach_with_hasher<N, FN, IN, H>(start: N, successors: FN, hasher: H) -> BfsReachable<N, FN, H>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    H: BuildHasher,
{
    let mut seen = IndexSet::with_hasher(hasher);
    seen.insert(start);
    BfsReachable {
        i: 0,
        seen,
        successors,
    }
}

/// Struct returned by [`bfs_reach`].
pub struct BfsReachable<N, FN, H> {
    i: usize,
    seen: IndexSet<N, H>,
    successors: FN,
}

impl<N, FN, H> BfsReachable<N, FN, H> {
    /// Return a lower bound on the number of remaining reachable
    /// nodes. Not all nodes are necessarily known in advance, and
    /// new reachable nodes may be discovered while using the iterator.
    pub fn remaining_nodes_low_bound(&self) -> usize {
        self.seen.len() - self.i
    }
}

impl<N, FN, IN, H> Iterator for BfsReachable<N, FN, H>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    H: BuildHasher,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.seen.get_index(self.i)?.clone();
        for s in (self.successors)(&n) {
            self.seen.insert(s);
        }
        self.i += 1;
        Some(n)
    }
}

impl<N, FN, IN, H> FusedIterator for BfsReachable<N, FN, H>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    H: BuildHasher,
{
}
