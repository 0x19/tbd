/// Time as a dependency, so a token's expiry or a retry's backoff can be
/// tested without waiting for it.
abstract interface class Clock {
  /// The current instant, UTC.
  DateTime now();
}

/// The wall clock; what an app uses.
final class SystemClock implements Clock {
  /// Stateless, so `const`.
  const SystemClock();

  @override
  DateTime now() => DateTime.now().toUtc();
}

/// A clock a test moves by hand.
final class FixedClock implements Clock {
  /// Starts at [_now] and moves only through [advance].
  FixedClock(this._now);
  DateTime _now;

  @override
  DateTime now() => _now;

  /// Move the clock forward [by].
  void advance(Duration by) => _now = _now.add(by);
}
