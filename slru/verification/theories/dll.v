From radium Require Import lang notation.
From refinedrust Require Import typing shims.

(* doubly-linked list that refines [LruCache].

    A cache holding [l = [x0; ..; x_{n-1}]] is laid out as

      hd  <->  m0  <->  ..  <->  m_{n-1}  <->  tl

    with 
    [hd.prev = NULL] 
    [tl.next = NULL]. 
    [locs = [m0; ..; m_{n-1}]]: the list of payload node addresses. 

    [dll_chain] is the same sequence with both sentinels spliced in, so payload
    index [i] sits at chain index [S i] and a payload node's neighbours are
    always [C !!! i] and [C !!! S (S i)] -- no special case at either end. The
    [big_sepL] itself ranges over the refinement list [l], so keys and values
    are read straight off [l] with no index shifting. 
*)

(* generic helpers *)

Lemma delete_app_l {A} (k1 k2 : list A) (i : nat) :
  (i < length k1)%nat → delete i (k1 ++ k2) = delete i k1 ++ k2.
Proof.
  intros Hi. rewrite !delete_take_drop take_app_le; last lia.
  rewrite drop_app_le; last lia. by rewrite app_assoc.
Qed.

Section big_sepL_aux.
  Context {PROP : bi} {A : Type}.
  Implicit Types (k : list A) (Φ : nat → A → PROP).

  (* open [k] at index [i], keeping prefix and suffix as separate segments.
     stated in both directions so it can be used with [iDestruct] / [iApply]. *)
  Lemma big_sepL_split_middle_1 Φ k (i : nat) x :
    k !! i = Some x →
    ([∗ list] j ↦ y ∈ k, Φ j y) -∗
      ([∗ list] j ↦ y ∈ take i k, Φ j y) ∗ Φ i x ∗
      ([∗ list] j ↦ y ∈ drop (S i) k, Φ (S i + j)%nat y).
  Proof.
    intros Hi. apply lookup_lt_Some in Hi as Hlt.
    rewrite -{1}(take_drop_middle k i x Hi).
    rewrite big_sepL_app big_sepL_cons length_take_le; last lia.
    rewrite Nat.add_0_r. setoid_rewrite Nat.add_succ_r. by iIntros "($ & $ & $)".
  Qed.

  Lemma big_sepL_split_middle_2 Φ k (i : nat) x :
    k !! i = Some x →
    ([∗ list] j ↦ y ∈ take i k, Φ j y) -∗ Φ i x -∗
    ([∗ list] j ↦ y ∈ drop (S i) k, Φ (S i + j)%nat y) -∗
    ([∗ list] j ↦ y ∈ k, Φ j y).
  Proof.
    intros Hi. apply lookup_lt_Some in Hi as Hlt.
    rewrite -{3}(take_drop_middle k i x Hi).
    rewrite big_sepL_app big_sepL_cons length_take_le; last lia.
    rewrite Nat.add_0_r. setoid_rewrite Nat.add_succ_r. iIntros "H1 H2 H3". iFrame.
  Qed.

  (* [delete i k] is the same two segments, with the suffix shifted down by one. *)
  Lemma big_sepL_split_delete_2 Φ k (i : nat) :
    (i ≤ length k)%nat →
    ([∗ list] j ↦ y ∈ take i k, Φ j y) -∗
    ([∗ list] j ↦ y ∈ drop (S i) k, Φ (i + j)%nat y) -∗
    ([∗ list] j ↦ y ∈ delete i k, Φ j y).
  Proof.
    intros Hi. rewrite delete_take_drop big_sepL_app length_take_le; last lia.
    iIntros "$ $".
  Qed.

  (* peel the last element off a prefix. *)
  Lemma big_sepL_take_S_1 Φ k (i : nat) z :
    k !! i = Some z →
    ([∗ list] j ↦ y ∈ take (S i) k, Φ j y) -∗
      ([∗ list] j ↦ y ∈ take i k, Φ j y) ∗ Φ i z.
  Proof.
    intros Hi. apply lookup_lt_Some in Hi as Hlt.
    rewrite (take_S_r _ _ _ Hi) big_sepL_app big_sepL_singleton length_take_le; last lia.
    rewrite Nat.add_0_r. by iIntros "($ & $)".
  Qed.

  Lemma big_sepL_take_S_2 Φ k (i : nat) z :
    k !! i = Some z →
    ([∗ list] j ↦ y ∈ take i k, Φ j y) -∗ Φ i z -∗
      ([∗ list] j ↦ y ∈ take (S i) k, Φ j y).
  Proof.
    intros Hi. apply lookup_lt_Some in Hi as Hlt.
    rewrite (take_S_r _ _ _ Hi) big_sepL_app big_sepL_singleton length_take_le; last lia.
    rewrite Nat.add_0_r. iIntros "H1 H2". iFrame.
  Qed.
End big_sepL_aux.

Section slru_dll.
  Context `{RRGS : !refinedrustGS Σ}.
  Context {K_rt V_rt : RT}.

  (* 
    ownership of a single node; created at the struct invariant with a
    place assertion for [ListNode]. keeping it abstract here forces
    every surgery lemma below into accessor form: a lemma can never change the
    links recorded in a node it does not hand out. 
  *)
  Context (node_own :
    thread_id → loc →
    option (place_rfn K_rt) → option (place_rfn V_rt) →
    loc →                        (* prev *)
    loc →                        (* next *)
    iProp Σ).

  Implicit Types (hd tl : loc) (locs : list loc) (l : list (RT_xt K_rt * RT_xt V_rt)).

  Definition dll_chain hd locs tl : list loc := hd :: locs ++ [tl].

  (* [length locs = length l] and non-nullness of the chain are
     [#[rr::invariant]] clauses on the struct, so Lithium sees them as ordinary
     pure hypotheses rather than having to dig them out of an [iProp]. *)
  Definition slru_dll π hd tl locs l : iProp Σ :=
    node_own π hd None None NULL_loc (dll_chain hd locs tl !!! (1%nat)) ∗
    node_own π tl None None (dll_chain hd locs tl !!! length l) NULL_loc ∗
    ([∗ list] i ↦ x ∈ l,
       node_own π (dll_chain hd locs tl !!! S i)
         (Some (# ($# x.1))) (Some (# ($# x.2)))
         (dll_chain hd locs tl !!! i) (dll_chain hd locs tl !!! S (S i))).
End slru_dll.

(* let resolution and Lithium see through these. In particular
   [ex_plain_t_solve_shr_auto] discharges the invariant's [Shareable] /
   [GhostDroppable] obligations by [apply _], which needs to reach the
   [∗] / [big_sepL] / [guarded] skeleton. *)
Global Typeclasses Transparent slru_dll dll_chain.
Global Hint Unfold slru_dll dll_chain : tyunfold.

Section lemmas.
  Context `{RRGS : !refinedrustGS Σ}.
  Context {K_rt V_rt : RT}.
  Context (node_own :
    thread_id → loc →
    option (place_rfn K_rt) → option (place_rfn V_rt) →
    loc → loc → iProp Σ).

  Implicit Types (hd tl : loc) (locs : list loc) (l : list (RT_xt K_rt * RT_xt V_rt)).

  (* chain arithmetic *)

  Lemma dll_chain_length hd locs tl :
    length (dll_chain hd locs tl) = S (S (length locs)).
  Proof. rewrite /dll_chain /= length_app /=. lia. Qed.

  Lemma dll_chain_lookup_0 hd locs tl : dll_chain hd locs tl !!! (0%nat) = hd.
  Proof. done. Qed.

  Lemma dll_chain_lookup_payload hd locs tl (i : nat) :
    (i < length locs)%nat → dll_chain hd locs tl !!! S i = locs !!! i.
  Proof. intros ?. rewrite /dll_chain !list_lookup_total_alt /= lookup_app_l //. Qed.

  Lemma dll_chain_lookup_tail hd locs tl :
    dll_chain hd locs tl !!! S (length locs) = tl.
  Proof.
    rewrite /dll_chain list_lookup_total_alt /= lookup_app_r; last lia.
    by rewrite Nat.sub_diag.
  Qed.

  (* [find] starts at [self.head.next]: the first payload node, or [tl]. *)
  Lemma dll_chain_head_next hd locs tl :
    dll_chain hd locs tl !!! (1%nat) = default tl (head locs).
  Proof. by destruct locs. Qed.

  (* the eviction victim in the third branch of [put] is [tl.prev]. *)
  Lemma dll_chain_tail_prev hd locs tl :
    dll_chain hd locs tl !!! length locs = default hd (last locs).
  Proof.
    destruct (length locs) as [|m] eqn:Hn.
    { apply nil_length_inv in Hn as ->. done. }
    rewrite dll_chain_lookup_payload; last lia.
    rewrite last_lookup Hn /=.
    destruct (locs !! m) as [z|] eqn:Hz.
    - by rewrite list_lookup_total_alt Hz.
    - apply lookup_ge_None_1 in Hz. lia.
  Qed.

  (* detaching a payload node removes exactly one entry from the chain. *)
  Lemma dll_chain_delete hd locs tl (i : nat) :
    (i < length locs)%nat →
    dll_chain hd (delete i locs) tl = delete (S i) (dll_chain hd locs tl).
  Proof. intros ?. rewrite /dll_chain /=. f_equal. by rewrite delete_app_l. Qed.

  (* attaching at the front shifts every existing chain position up by one. *)
  Lemma dll_chain_cons hd ln locs tl (j : nat) :
    dll_chain hd (ln :: locs) tl !!! S (S j) = dll_chain hd locs tl !!! S j.
  Proof. done. Qed.

  Lemma dll_chain_cons_1 hd ln locs tl :
    dll_chain hd (ln :: locs) tl !!! (1%nat) = ln.
  Proof. done. Qed.

  (* [LruCache::new] *)

  Lemma slru_dll_nil π hd tl :
    node_own π hd None None NULL_loc tl ∗
    node_own π tl None None hd NULL_loc
    ⊢ slru_dll node_own π hd tl [] [].
  Proof. rewrite /slru_dll /dll_chain /=. by iIntros "($ & $)". Qed.

  (*
      Every lemma here is an accessor: it hands out exactly the nodes the
      corresponding Rust statement writes to, and takes them back with the new
      links in place. The pointer writes themselves happen in the client proof,
      where [node_own] is a concrete place assertion.

      There is deliberately no fused "touch" / "move to front" lemma. [get] and
      [put] compose [slru_dll_detach] with [slru_dll_attach_front]
      *sequentially*: the detach closer restores the whole invariant before the
      attach accessor opens it again, so the aliasing cases a fused five-node
      lemma would have to enumerate (i = 0, i = 1) never arise. *)

  (* update the payload at index [i] without touching a single link: [get]'s
     return value, and the [mem::swap] in the hit branch of [put]. *)
  Lemma slru_dll_acc π hd tl locs l (i : nat) x :
    l !! i = Some x →
    slru_dll node_own π hd tl locs l -∗
    node_own π (dll_chain hd locs tl !!! S i)
             (Some (# ($# x.1))) (Some (# ($# x.2)))
             (dll_chain hd locs tl !!! i) (dll_chain hd locs tl !!! S (S i)) ∗
    (∀ x', node_own π (dll_chain hd locs tl !!! S i)
             (Some (# ($# x'.1))) (Some (# ($# x'.2)))
             (dll_chain hd locs tl !!! i) (dll_chain hd locs tl !!! S (S i)) -∗
           slru_dll node_own π hd tl locs (<[i := x']> l)).
  Proof.
    iIntros (Hi) "(Hhd & Htl & Hpay)".
    iDestruct (big_sepL_insert_acc _ _ i with "Hpay") as "($ & Hcl)"; first done.
    iIntros (x') "Hn". rewrite /slru_dll length_insert. iFrame "Hhd Htl".
    by iApply "Hcl".
  Qed.

  (* [detach] at payload index [i]. The caller gets the node itself -- it keeps
     it, since both [get] and [put] re-attach it -- together with its two chain
     neighbours, and hands the neighbours back with [prev.next] and [next.prev]
     repointed past the removed node.

     The neighbours may be sentinels ([i = 0] / [S i = length l]); the statement
     is uniform in that, only the proof splits. *)
  Lemma slru_dll_detach π hd tl locs l (i : nat) x :
    length locs = length l → l !! i = Some x →
    slru_dll node_own π hd tl locs l -∗
    ∃ kp vp pp kq vq qn,
      node_own π (dll_chain hd locs tl !!! i) kp vp pp
               (dll_chain hd locs tl !!! S i) ∗
      node_own π (dll_chain hd locs tl !!! S i)
               (Some (# ($# x.1))) (Some (# ($# x.2)))
               (dll_chain hd locs tl !!! i) (dll_chain hd locs tl !!! S (S i)) ∗
      node_own π (dll_chain hd locs tl !!! S (S i)) kq vq
               (dll_chain hd locs tl !!! S i) qn ∗
      (node_own π (dll_chain hd locs tl !!! i) kp vp pp
                (dll_chain hd locs tl !!! S (S i)) -∗
       node_own π (dll_chain hd locs tl !!! S (S i)) kq vq
                (dll_chain hd locs tl !!! i) qn -∗
       slru_dll node_own π hd tl (delete i locs) (delete i l)).
  Proof.
    iIntros (Hlen Hi) "(Hhd & Htl & Hpay)".
    apply lookup_lt_Some in Hi as Hlt.
    set (C := dll_chain hd locs tl).
    set (C' := dll_chain hd (delete i locs) tl).
    assert (HCC' : C' = delete (S i) C).
    { subst C C'. apply dll_chain_delete. lia. }
    assert (Hlo : ∀ j, (j ≤ i)%nat → C' !!! j = C !!! j).
    { intros j ?. rewrite HCC' list_lookup_total_delete_lt //. lia. }
    assert (Hhi : ∀ j, (S i ≤ j)%nat → C' !!! j = C !!! S j).
    { intros j ?. by rewrite HCC' list_lookup_total_delete_ge. }
    assert (Hdl : length (delete i l) = pred (length l)).
    { rewrite length_delete; [lia | by eexists]. }
    iDestruct (big_sepL_split_middle_1 _ _ i x with "Hpay") as "(HA & Hnode & HB)";
      first done.

    (* --- the predecessor, together with a closer for the head sentinel and
           the untouched prefix of the payload --- *)
    iAssert (∃ kp vp pp,
      node_own π (C !!! i) kp vp pp (C !!! S i) ∗
      (node_own π (C !!! i) kp vp pp (C !!! S (S i)) -∗
         node_own π hd None None NULL_loc (C' !!! (1%nat)) ∗
         ([∗ list] j ↦ y ∈ take i l,
            node_own π (C' !!! S j) (Some (# ($# y.1))) (Some (# ($# y.2)))
                     (C' !!! j) (C' !!! S (S j)))))%I
      with "[Hhd HA]" as (kp vp pp) "(Hp & Hpcl)".
    { destruct i as [|i0].
      - (* the predecessor is the head sentinel *)
        iExists None, None, NULL_loc. rewrite dll_chain_lookup_0. iFrame "Hhd".
        iIntros "Hhd". rewrite (Hhi 1%nat); last lia. by iFrame "Hhd".
      - (* the predecessor is payload node [i0] *)
        destruct (l !! i0) as [z|] eqn:Hz; last (apply lookup_ge_None_1 in Hz; lia).
        iDestruct (big_sepL_take_S_1 _ _ i0 z with "HA") as "(HA & Hz)"; first done.
        iExists _, _, _. iFrame "Hz".
        iIntros "Hz". rewrite (Hlo 1%nat); last lia. iFrame "Hhd".
        iApply (big_sepL_take_S_2 _ _ i0 z with "[HA] [Hz]"); first done.
        + iApply (big_sepL_proper with "HA"). intros j y Hy%lookup_lt_Some.
          rewrite length_take in Hy.
          rewrite !Hlo //; lia.
        + rewrite (Hlo (S i0)) //. rewrite (Hlo i0); last lia.
          rewrite (Hhi (S (S i0))) //. }

    (* --- the successor, together with a closer for the tail sentinel and the
           untouched suffix of the payload --- *)
    iAssert (∃ kq vq qn,
      node_own π (C !!! S (S i)) kq vq (C !!! S i) qn ∗
      (node_own π (C !!! S (S i)) kq vq (C !!! i) qn -∗
         node_own π tl None None (C' !!! length (delete i l)) NULL_loc ∗
         ([∗ list] j ↦ y ∈ drop (S i) l,
            node_own π (C' !!! S (i + j)) (Some (# ($# y.1))) (Some (# ($# y.2)))
                     (C' !!! (i + j)%nat) (C' !!! S (S (i + j))))))%I
      with "[Htl HB]" as (kq vq qn) "(Hq & Hqcl)".
    { destruct (drop (S i) l) as [|z rest] eqn:Hd.
      - (* the successor is the tail sentinel *)
        assert (length l = S i) as Hn.
        { apply (f_equal length) in Hd. rewrite length_drop /= in Hd. lia. }
        iExists None, None, NULL_loc.
        assert (C !!! S (S i) = tl) as ->.
        { subst C. rewrite -Hn -Hlen dll_chain_lookup_tail //. }
        rewrite -Hn. iFrame "Htl".
        iIntros "Htl". rewrite Hdl Hn /= (Hlo i) //. by iFrame "Htl".
      - (* the successor is payload node [S i] *)
        rewrite big_sepL_cons Nat.add_0_r.
        iDestruct "HB" as "(Hz & HB)".
        assert (S i < length l)%nat as Hsi.
        { apply (f_equal length) in Hd. rewrite length_drop /= in Hd. lia. }
        iExists _, _, _. iFrame "Hz".
        iIntros "Hz". rewrite Hdl (Hhi (pred (length l))); last lia. rewrite -Hlen.
        replace (S (pred (length locs))) with (length locs) by lia.
        iFrame "Htl".
        rewrite big_sepL_cons Nat.add_0_r.
        rewrite (Hhi (S i)); try lia. rewrite (Hlo i); try lia.
        rewrite (Hhi (S (S i))); try lia.
        iFrame "Hz".
        iApply (big_sepL_proper with "HB"). intros j y _.
        rewrite !Hhi; try lia. done. }

    iExists kp, vp, pp, kq, vq, qn. iFrame "Hp Hnode Hq".
    iIntros "Hp Hq".
    iDestruct ("Hpcl" with "Hp") as "(Hhd & HA)".
    iDestruct ("Hqcl" with "Hq") as "(Htl & HB)".
    rewrite /slru_dll -/C'. iFrame "Hhd Htl".
    iApply (big_sepL_split_delete_2 with "HA HB"). lia.
  Qed.

  (* [head.attach(node)]: splice [ln] in at the MRU end. The caller supplies the
     node -- freshly boxed, or just detached -- and gets back the head sentinel
     and the current first node, whose links it must patch. [ln]'s non-nullness
     is a struct-level invariant, re-established by the client with
     [Forall_cons]. *)
  Lemma slru_dll_attach_front π hd tl locs l ln x :
    length locs = length l →
    slru_dll node_own π hd tl locs l -∗
    ∃ kf vf fn,
      node_own π hd None None NULL_loc (dll_chain hd locs tl !!! (1%nat)) ∗
      node_own π (dll_chain hd locs tl !!! (1%nat)) kf vf hd fn ∗
      (node_own π hd None None NULL_loc ln -∗
       node_own π ln (Some (# ($# x.1))) (Some (# ($# x.2))) hd
                (dll_chain hd locs tl !!! (1%nat)) -∗
       node_own π (dll_chain hd locs tl !!! (1%nat)) kf vf ln fn -∗
       slru_dll node_own π hd tl (ln :: locs) (x :: l)).
  Proof.
    iIntros (Hlen) "(Hhd & Htl & Hpay)".
    set (C := dll_chain hd locs tl).
    destruct l as [|y l'].
    - (* the "first node" is the tail sentinel *)
      assert (locs = []) as -> by (apply nil_length_inv; by rewrite Hlen).
      iExists None, None, NULL_loc. iFrame "Hhd Htl".
      iIntros "Hhd Hn Htl". rewrite /slru_dll /dll_chain /=. iFrame.
    - (* the "first node" is payload node [0] *)
      rewrite big_sepL_cons. iDestruct "Hpay" as "(Hf & Hpay)".
      rewrite dll_chain_lookup_0.
      iExists _, _, _. iFrame "Hhd Hf".
      iIntros "Hhd Hn Hf". rewrite /slru_dll.
      rewrite dll_chain_cons_1.
      iFrame "Hhd".
      (* the tail sentinel's [prev] moves up one chain position *)
      rewrite {1}(_ : length (x :: y :: l') = S (length (y :: l'))); last done.
      rewrite dll_chain_cons. iFrame "Htl".
      rewrite big_sepL_cons dll_chain_lookup_0 dll_chain_cons_1 dll_chain_cons.
      iFrame "Hn".
      rewrite big_sepL_cons dll_chain_cons_1 !dll_chain_cons.
      iFrame "Hf".
      iApply (big_sepL_proper with "Hpay"). intros j z _.
      by rewrite !dll_chain_cons.
  Qed.

  (* [Drop] is still [#[rr::skip]] and cannot be verified yet: the invariant
     carries no [freeable_nz] per node, so [Box::from_raw] is unjustified. The
     prerequisite is to add
       freeable_nz ln (ly_size (use_struct_layout_alg' (ListNode_sls ..))) 1 HeapAlloc
     to the [#iris] invariant, as case_studies/linkedlist does, and only then
     add the chain-walk lemmas. *)

End lemmas.
