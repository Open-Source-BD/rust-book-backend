/* Rust Backend for Humans — retention widget (quiz + flashcards).
 *
 * Reads window.RUST_QUESTIONS (bundled by tools/generate.mjs). No dependencies,
 * no network. Progress is stored in localStorage so it survives reloads and
 * works offline.
 *
 *   Quiz:       <div class="quiz" data-topic="ownership"></div>   (on lesson pages)
 *   Flashcards: <div id="flashcards" data-review="all"></div>     (on review.md)
 */
(function () {
  "use strict";

  var Q = window.RUST_QUESTIONS || {};
  var ORDER = window.RUST_TOPIC_ORDER || [];
  var LS = window.localStorage;
  var KEY = "rbh:v1:"; // bump to reset everyone's saved state

  // ---- storage helpers -----------------------------------------------------
  function get(k, dflt) {
    try {
      var v = LS.getItem(KEY + k);
      return v === null ? dflt : JSON.parse(v);
    } catch (e) {
      return dflt;
    }
  }
  function set(k, v) {
    try {
      LS.setItem(KEY + k, JSON.stringify(v));
    } catch (e) {}
  }

  function el(tag, cls, html) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (html != null) n.innerHTML = html;
    return n;
  }
  function esc(s) {
    return String(s == null ? "" : s).replace(/[&<>"]/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c];
    });
  }
  function shuffle(a) {
    for (var i = a.length - 1; i > 0; i--) {
      var j = Math.floor(Math.random() * (i + 1));
      var t = a[i];
      a[i] = a[j];
      a[j] = t;
    }
    return a;
  }

  // =========================================================================
  // QUIZ  (end of each lesson)
  // =========================================================================
  function renderQuiz(host) {
    var topic = host.getAttribute("data-topic");
    var bank = Q[topic];
    var quiz = bank && bank.quiz ? bank.quiz : [];
    host.innerHTML = "";
    host.classList.add("rfh-quiz");

    if (!quiz.length) {
      host.appendChild(
        el(
          "p",
          "rfh-muted",
          "🛈 Quick-check questions for this topic are coming soon."
        )
      );
      return;
    }

    var state = get("quiz:" + topic, {}); // { qIndex: chosenIndex }
    var head = el("div", "rfh-quiz-head");
    var score = el("span", "rfh-score", "");
    head.appendChild(el("strong", null, "Quick check"));
    head.appendChild(score);
    host.appendChild(head);

    function refreshScore() {
      var done = 0,
        right = 0;
      quiz.forEach(function (q, i) {
        if (state[i] != null) {
          done++;
          if (state[i] === q.answer) right++;
        }
      });
      score.textContent = done ? right + " / " + quiz.length + " correct" : "";
      if (done === quiz.length && right === quiz.length) {
        score.classList.add("rfh-score-perfect");
      } else {
        score.classList.remove("rfh-score-perfect");
      }
    }

    quiz.forEach(function (q, i) {
      var card = el("div", "rfh-q");
      card.appendChild(el("p", "rfh-q-text", esc(q.q)));
      var opts = el("div", "rfh-opts");
      var explain = el("div", "rfh-explain");

      (q.options || []).forEach(function (opt, oi) {
        var b = el("button", "rfh-opt", esc(opt));
        b.type = "button";
        function paint() {
          var chosen = state[i];
          b.classList.remove("rfh-correct", "rfh-wrong", "rfh-chosen");
          if (chosen == null) return;
          if (oi === q.answer) b.classList.add("rfh-correct");
          if (oi === chosen && chosen !== q.answer) b.classList.add("rfh-wrong");
          if (oi === chosen) b.classList.add("rfh-chosen");
          b.disabled = true;
        }
        b.addEventListener("click", function () {
          if (state[i] != null) return; // already answered
          state[i] = oi;
          set("quiz:" + topic, state);
          Array.prototype.forEach.call(opts.children, function (c) {
            if (c._paint) c._paint();
          });
          explain.innerHTML =
            (oi === q.answer ? "✅ Correct. " : "❌ Not quite. ") +
            (q.explain ? esc(q.explain) : "");
          explain.classList.add("rfh-explain-show");
          refreshScore();
        });
        b._paint = paint;
        paint();
        opts.appendChild(b);
      });

      card.appendChild(opts);
      // restore explanation if already answered
      if (state[i] != null) {
        explain.innerHTML =
          (state[i] === q.answer ? "✅ Correct. " : "❌ Not quite. ") +
          (q.explain ? esc(q.explain) : "");
        explain.classList.add("rfh-explain-show");
      }
      card.appendChild(explain);
      host.appendChild(card);
    });

    var reset = el("button", "rfh-reset", "Reset quiz");
    reset.type = "button";
    reset.addEventListener("click", function () {
      set("quiz:" + topic, {});
      renderQuiz(host);
    });
    host.appendChild(reset);
    refreshScore();
  }

  // =========================================================================
  // FLASHCARDS  (review page)
  // =========================================================================
  function buildDeck() {
    var cards = [];
    ORDER.forEach(function (meta) {
      var bank = Q[meta.slug];
      if (!bank) return;
      (bank.flashcards || []).forEach(function (c, i) {
        cards.push({
          id: meta.slug + ":fc:" + i,
          topic: meta.slug,
          title: meta.title,
          front: c.front,
          back: c.back,
        });
      });
      // Quiz questions double as flashcards (write once, review twice).
      (bank.quiz || []).forEach(function (q, i) {
        var ans = (q.options || [])[q.answer];
        cards.push({
          id: meta.slug + ":qz:" + i,
          topic: meta.slug,
          title: meta.title,
          front: q.q,
          back: (ans ? ans + ". " : "") + (q.explain || ""),
        });
      });
    });
    return cards;
  }

  function renderFlashcards(host) {
    var all = buildDeck();
    host.innerHTML = "";
    host.classList.add("rfh-review");

    if (!all.length) {
      host.appendChild(
        el(
          "p",
          "rfh-muted",
          "🛈 No cards yet. As topics get quiz questions and flashcards, they show up here automatically."
        )
      );
      return;
    }

    var marks = get("cards", {}); // { id: "known" | "shaky" }
    var filterShaky = false;

    // Controls
    var bar = el("div", "rfh-review-bar");
    var counter = el("span", "rfh-review-count", "");
    var toggle = el("button", "rfh-btn", "Show shaky only");
    toggle.type = "button";
    var restart = el("button", "rfh-btn", "Shuffle & restart");
    restart.type = "button";
    bar.appendChild(counter);
    bar.appendChild(toggle);
    bar.appendChild(restart);
    host.appendChild(bar);

    var stage = el("div", "rfh-stage");
    host.appendChild(stage);

    var queue = [];
    var pos = 0;

    function order() {
      // Shaky first, then unseen, then known. Optionally shaky-only.
      var pool = all.filter(function (c) {
        return filterShaky ? marks[c.id] === "shaky" : true;
      });
      var shaky = [],
        unseen = [],
        known = [];
      pool.forEach(function (c) {
        if (marks[c.id] === "shaky") shaky.push(c);
        else if (marks[c.id] === "known") known.push(c);
        else unseen.push(c);
      });
      return shuffle(shaky).concat(shuffle(unseen)).concat(shuffle(known));
    }

    function stats() {
      var known = 0,
        shaky = 0;
      all.forEach(function (c) {
        if (marks[c.id] === "known") known++;
        else if (marks[c.id] === "shaky") shaky++;
      });
      return { known: known, shaky: shaky, total: all.length };
    }

    function showDone() {
      stage.innerHTML = "";
      var s = stats();
      var done = el("div", "rfh-done");
      done.appendChild(el("p", null, "🎉 Deck complete for this round."));
      done.appendChild(
        el(
          "p",
          "rfh-muted",
          s.known + " known · " + s.shaky + " shaky · " + s.total + " total"
        )
      );
      var again = el("button", "rfh-btn rfh-btn-primary", "Go again");
      again.type = "button";
      again.addEventListener("click", start);
      done.appendChild(again);
      stage.appendChild(done);
    }

    function render() {
      if (pos >= queue.length) return showDone();
      var c = queue[pos];
      var s = stats();
      counter.textContent =
        "Card " +
        (pos + 1) +
        " / " +
        queue.length +
        "  ·  " +
        s.known +
        " known, " +
        s.shaky +
        " shaky";

      stage.innerHTML = "";
      var card = el("div", "rfh-card");
      card.appendChild(el("span", "rfh-card-topic", esc(c.title)));
      card.appendChild(el("div", "rfh-card-front", esc(c.front)));

      var back = el("div", "rfh-card-back", esc(c.back || "(no answer text)"));
      back.style.display = "none";
      card.appendChild(back);

      var flip = el("button", "rfh-btn rfh-btn-primary rfh-flip", "Flip");
      flip.type = "button";

      var actions = el("div", "rfh-card-actions");
      actions.style.display = "none";
      var gotIt = el("button", "rfh-btn rfh-good", "Got it");
      var shakyBtn = el("button", "rfh-btn rfh-bad", "Shaky");
      gotIt.type = shakyBtn.type = "button";
      actions.appendChild(shakyBtn);
      actions.appendChild(gotIt);

      flip.addEventListener("click", function () {
        back.style.display = "";
        flip.style.display = "none";
        actions.style.display = "";
      });
      function mark(v) {
        marks[c.id] = v;
        set("cards", marks);
        pos++;
        render();
      }
      gotIt.addEventListener("click", function () {
        mark("known");
      });
      shakyBtn.addEventListener("click", function () {
        mark("shaky");
      });

      card.appendChild(flip);
      card.appendChild(actions);
      stage.appendChild(card);
    }

    function start() {
      queue = order();
      pos = 0;
      if (!queue.length) {
        stage.innerHTML = "";
        stage.appendChild(
          el("p", "rfh-muted", "Nothing to review here right now.")
        );
        return;
      }
      render();
    }

    toggle.addEventListener("click", function () {
      filterShaky = !filterShaky;
      toggle.textContent = filterShaky ? "Show all cards" : "Show shaky only";
      toggle.classList.toggle("rfh-btn-active", filterShaky);
      start();
    });
    restart.addEventListener("click", start);

    start();
  }

  // ---- boot ----------------------------------------------------------------
  function init() {
    document.querySelectorAll(".quiz[data-topic]").forEach(renderQuiz);
    var review = document.getElementById("flashcards");
    if (review) renderFlashcards(review);
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
