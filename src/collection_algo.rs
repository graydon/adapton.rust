#![feature(associated_type_defaults)]
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
    
use macros::* ;
use adapton_sigs::* ;
use collection_traits::*;
use quickcheck::Arbitrary;
use quickcheck::Gen;
use std::num::Zero;

use rand::{Rng,Rand};

pub fn tree_reduce_monoid<A:Adapton,Elm:Eq+Hash+Clone+Debug,T:TreeT<A,Elm>,BinOp>
    (st:&mut A, tree:T::Tree, zero:Elm, binop:&BinOp) -> Elm
    where BinOp:Fn(&mut A, Elm, Elm) -> Elm
{
    T::fold_up(st, tree,
                        &|_| zero.clone(),
                   &|_,leaf| leaf,
                 &|st,_,l,r| binop(st,l,r),
               &|st,_,_,l,r| binop(st,l,r),
               )
}

pub fn list_reduce_monoid<A:Adapton,Elm:Eq+Hash+Clone+Debug,L:ListT<A,Elm>,BinOp,T:TreeT<A,Elm>>
    (st:&mut A, list:L::List, zero:Elm, binop:&BinOp) -> Elm
    where BinOp:Fn(&mut A, Elm, Elm) -> Elm
{
    let tree = tree_of_list::<A,Elm,T,L>(st, Dir2::Left, list);
    tree_reduce_monoid::<A,Elm,T,BinOp>(st, tree, zero, binop)
}

// pub fn tree_map<A:Adapton,X:Hash+Clone,T:TreeT<A,X>,FX:Hash+Clone,GY:Hash+Clone,FGT:TreeT<A,FX,GY>,F,G>
//     (st:&mut A, tree:T::Tree, f:&F, g:&G) -> FGT::Tree
//     where F:Fn(&mut A, X) -> FX
//     ,     G:Fn(&mut A, Y) -> GY
// {
//     T::fold_up(st, tree,
//                &|st|       FGT::nil(st),
//                &|st,x|     {let fx = f(st,x);  FGT::leaf(st, fx)},
//                &|st,y,l,r| {let gy = g(st, y); FGT::bin(st, gy, l, r)},
//                &|st,n,l,r| FGT::name(st, n, l, r)
//                )
// }

pub fn tree_filter<A:Adapton,X:Hash+Clone,T:TreeT<A,X>,F>
    (st:&mut A, tree:T::Tree, f:&F) -> T::Tree
    where F:Fn(&mut A, &X) -> bool
{
    T::fold_up(st, tree,
               &|st| T::nil(st),
               &|st,x| { let fx = f(st,&x);
                         if fx { T::leaf(st, x) }
                         else  { T::nil(st) } },
               &|st,lev,l,r| T::bin(st, lev, l, r),
               &|st,n,lev,l,r| T::name(st, n, lev, l, r)
               )
}

pub fn list_of_tree<A:Adapton,X:Hash+Clone,L:ListT<A,X>,T:TreeT<A,X>>
    (st:&mut A, tree:T::Tree) -> L::List
{
    let nil = L::nil(st);
    T::fold_rl(st, tree, nil,
               &|st,x,xs| L::cons(st,x,xs),
               &|_,_,xs| xs,
               &|st,n,_,xs| L::name(st,n,xs)
               )
}

pub fn rev_list_of_tree<A:Adapton,X:Hash+Clone,L:ListT<A,X>,T:TreeT<A,X>>
    (st:&mut A, tree:T::Tree) -> L::List
{
    let nil = L::nil(st);
    T::fold_lr(st, tree, nil,
               &|st,x,xs| L::cons(st,x,xs),
               &|_ ,_,xs| xs,
               &|st,n,_,xs| L::name(st,n,xs)
               )    
}

pub fn list_of_vec<A:Adapton,X:Clone,L:ListT<A,X>> (st:&mut A, v:Vec<X>) -> L::List {
    let mut l = L::nil(st);
    for x in v.iter().rev() { l = L::cons(st,x.clone(),l) }
    return l
}

pub fn rev_list_of_vec<A:Adapton,X:Clone,L:ListT<A,X>> (st:&mut A, v:Vec<X>) -> L::List {
    let mut l = L::nil(st);
    for x in v.iter() { l = L::cons(st,x.clone(), l) }
    return l
}

pub fn tree_of_list
    < A:Adapton
    , X:Hash+Clone+Debug
    , T:TreeT<A,X>
    , L:ListT<A,X>
    >
    (st:&mut A, dir_list:Dir2, list:L::List) -> T::Tree {
        let tnil = T::nil(st);
        let lnil = L::nil(st);
        let (tree, list) = tree_of_list_rec::<A,X,T,L>(st, dir_list, list, tnil, T::lev_zero(), T::lev_max());
        assert_eq!(list, lnil);
        tree
    }

pub fn tree_of_list_rec
    < A:Adapton
    , X:Hash+Clone+Debug
    , T:TreeT<A,X>
    , L:ListT<A,X>
    >
    (st:&mut A,
     dir_list:Dir2, list:L::List,
     tree:T::Tree, tree_lev:T::Lev, parent_lev:T::Lev)
     -> (T::Tree, L::List)
{
    L::elim_move (
        st, list, (dir_list, tree, tree_lev, parent_lev),

        /* Nil */
        |st, (_dir_list, tree, _, _)| (tree, L::nil(st)),

        /* Cons */
        |st, hd, rest, (dir_list, tree, tree_lev, parent_lev)| {
            let lev_hd = T::lev_inc ( T::lev(&hd) ) ;
            if T::lev_lte ( tree_lev , lev_hd.clone() ) && T::lev_lte ( lev_hd.clone() , parent_lev.clone() ) {
                let leaf = T::leaf(st, hd) ;
                let (tree2, rest2) = {
                    tree_of_list_rec::<A,X,T,L> ( st, dir_list.clone(), rest, leaf, T::lev_zero(), lev_hd.clone() )
                };
                let tree3 = match dir_list.clone() {
                    Dir2::Left  => T::bin ( st, lev_hd.clone(), tree,  tree2 ),
                    Dir2::Right => T::bin ( st, lev_hd.clone(), tree2, tree  ),
                } ;
                tree_of_list_rec::<A,X,T,L> ( st, dir_list, rest2, tree3, lev_hd, parent_lev )
            }
            else {
                (tree, L::cons(st,hd,rest))
            }},

        /* Name */
        |st, nm, rest, (dir_list, tree, tree_lev, parent_lev)|{
            let lev_nm = T::lev_inc( T::lev_add( T::lev_bits() , T::lev(&nm) ) ) ;
            if T::lev_lte ( tree_lev , lev_nm.clone() ) && T::lev_lte ( lev_nm.clone() ,  parent_lev.clone() ) {
                let nil = T::nil(st) ;
                let (nm1, nm2, nm3, nm4) = st.name_fork4(nm.clone());
                let (_, (tree2, rest)) =
                    eager!(st, nm1 =>> tree_of_list_rec::<A,X,T,L>,
                           dir_list:dir_list.clone(), list:rest,
                           tree:nil, tree_lev:T::lev_zero(), parent_lev:lev_nm.clone() ) ;
                let tree3 = match dir_list.clone() {
                    Dir2::Left  => T::name ( st, nm.clone(), lev_nm.clone(), tree,  tree2 ),
                    Dir2::Right => T::name ( st, nm.clone(), lev_nm.clone(), tree2, tree  ),
                } ;
                let art = st.cell(nm3, tree3) ;
                let art = st.read_only( art ) ;
                let tree3 = T::art( st, art ) ;                
                let (_, (tree, rest)) =
                    eager!(st, nm2 =>> tree_of_list_rec::<A,X,T,L>,
                           dir_list:dir_list.clone(), list:rest,
                           tree:tree3, tree_lev:lev_nm, parent_lev:parent_lev ) ;
                let art = st.cell(nm4, tree) ;
                let art = st.read_only( art ) ;
                let tree = T::art( st, art ) ;
                (tree, rest)
            }
            else {
                (tree, L::name(st,nm,rest))
            }},
        )
}

pub fn list_merge<A:Adapton,X:Ord+Clone+Debug,L:ListT<A,X>>
    (st:&mut A,
     n1:Option<A::Name>, l1:L::List,
     n2:Option<A::Name>, l2:L::List  ) -> L::List
{
    L::elim_move
        (st, l1, (n1,n2,l2),
         /* Nil */  |_, (_, _, l2)| l2,
         /* Cons */ |st,h1,t1,(n1,n2,l2)|
         L::elim_move
         (st, l2, (h1,t1,n1,n2),
          /* Nil */  |st, (h1, t1, _, _ )| L::cons(st, h1,t1),
          /* Cons */ |st, h2, t2, (h1, t1, n1, n2)| {
          if &h1 <= &h2 {
              let l2 = L::cons(st,h2,t2);
              match n1 {
                  None => {
                      let rest = list_merge::<A,X,L>(st, None, t1, n2, l2);
                      L::cons(st, h1, rest)
                  }
                  Some(n1) => {
                      let (n1a, n1b) = st.name_fork(n1);
                      let rest = thunk!(st, n1a =>>
                                        list_merge::<A,X,L>,
                                        n1:None, l1:t1, n2:n2, l2:l2);
                      let rest = L::art(st, rest);
                      let rest = L::cons(st, h1, rest);
                      L::name(st, n1b, rest)
                  }
              }
          }
          else {
              let l1 = L::cons(st,h1,t1);
              match n2 {
                  None => {
                      let rest = list_merge::<A,X,L>(st, n1, l1, None, t2);
                      let l = L::cons(st, h2, rest);
                      l
                  }
                  Some(n2) => {
                      let (n2a, n2b) = st.name_fork(n2);
                      let rest = thunk!(st, n2a =>>
                                        list_merge::<A,X,L>,
                                        n1:n1, l1:l1, n2:None, l2:t2);
                      let rest = L::art(st, rest);
                      let rest = L::cons(st, h2, rest);
                      L::name(st, n2b, rest)
                  }
              }
          }},
          |st,m2,t2,(h1,t1,n1,_n2)| {
              let l1 = L::cons(st,h1,t1);
              list_merge::<A,X,L>(st, n1, l1, Some(m2), t2)
          }
          ),
         |st,m1,t1,(_n1,n2,l2)| {
             list_merge::<A,X,L>(st, Some(m1), t1, n2, l2)
         }
         )
}

pub fn list_merge_sort<A:Adapton,X:Ord+Hash+Debug+Clone,L:ListT<A,X>,T:TreeT<A,X>>
    (st:&mut A, list:L::List) -> L::List
{
    let tree = tree_of_list::<A,X,T,L>(st, Dir2::Left, list);
    T::fold_up (st, tree,
                &|st|                 L::nil(st), 
                &|st, x|              L::singleton(st, x),
                &|st, _, left, right| { list_merge::<A,X,L>(st, None, left, None, right) },
                &|st, n, _, left, right| { let (n1,n2) = st.name_fork(n);
                                           list_merge::<A,X,L>(st, Some(n1), left, Some(n2), right) },
                )
}

pub fn tree_append
    < A:Adapton
    , X:Hash+Clone+Debug
    , T:TreeT<A,X>
    >
    (st:&mut A,tree1:T::Tree,tree2:T::Tree) -> T::Tree
{
    // XXX: This is a hack. Make this balanced, a la Pugh 1989.
    T::bin(st, T::lev_max(), tree1, tree2)
}

// pub fn tree_append
//     <A:Adapton
//     ,X:Clone+Hash+Eq+Debug
//     ,T:TreeT<A,X,()>
//     >
//     (st:&mut A, tree1:T::Tree, tree2:T::Tree) -> T::Tree
// {
//     T::elim_move
//         (st, tree1, tree2,         
//          /* Nil */  |st,       tree2| tree2,
//          /* Leaf */ |st,leaf,  tree2| {
//              T::elim_move
//                  (st, tree2, leaf,
//                   /* Nil */  |st, leaf1| T::leaf(st,leaf1),
//                   /* Leaf */ |st, leaf2, leaf1| {
//                       let l1 = T::leaf(st,leaf1);
//                       let l2 = T::leaf(st,leaf2);
//                       T::bin(st, (), l1, l2)
//                   },
//                   /* Bin */  |st, l2, r2, leaf1| {
//                       let leaf = T::leaf(st, leaf);
//                       /*T::bin(st, (), leaf, tree2)*/
//                       panic!("TODO")
//                   },
//                   /* Name */ |st, nm, l2, r2, leaf1| {
//                       panic!("TODO")
//                   })
//          },
//          /* Bin */ |st,_,l,r, tree2| {
//              /*
//              T::elim_move
//                  (st, tree2, (l,r),
//                   /* Nil */ |st, 
                  
//                   let tree1 = T::bin(st, (), l, r);
//              T::bin(st, (), tree1, tree2)
//                  */
//                  panic!("")
//          },
//          /* Name */ |st,_,l,r, tree2| panic!(""),
//          )
// }