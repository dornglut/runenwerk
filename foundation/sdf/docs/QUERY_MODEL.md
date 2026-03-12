# Query Model

`foundation/sdf` provides query helpers that operate on `SdfField3`.

## Point Sampling

- `field.sample(point)` is the canonical entry point.
- Sampling is deterministic for fixed inputs and tolerances.

## Raymarch

`queries::raymarch::raymarch_first_hit`:

- advances along the ray direction by sampled distance
- reports a hit when distance is `<= epsilon`
- terminates when `t > max_distance` or `max_steps` reached

The method is sphere-tracing style; quality depends on how close the field is
 to a true signed-distance metric.

## Projection

`queries::project::project_point_to_surface`:

- iteratively moves by `-normal * distance`
- converges when `abs(distance) <= surface_epsilon`
- terminates on max iterations or low-quality gradient estimate

Useful for depenetration, grounding, and closest-surface style behavior.

## Classification

`queries::classify::classify_point` maps sample distance into:

- `Inside`
- `OnSurface`
- `Outside`

using an explicit epsilon threshold.

## Sweep Foundation

`queries::sweep::sweep_sphere` provides a first sweep/depenetration building
block by stepping a sphere center along a segment and testing field distance.
It is intentionally simple and can be replaced by higher quality methods later.
