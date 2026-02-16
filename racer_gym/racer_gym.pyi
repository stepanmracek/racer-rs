from enum import Enum
from typing import List, Optional, Tuple

class Action:
    steer: float
    throttle: float

    def __init__(self, steer: float, throttle: float) -> None: ...

class Goal(Enum):
    ReachFinish = ...
    BackToTrack = ...

class Environment:
    def __init__(
        self,
        seed: Optional[int] = 0,
        car_count: int = 1,
        off_track_prob: float = 0.0,
        goal: Goal = Goal.ReachFinish,
        track_width: float = 42.0,
    ) -> None: ...
    def step(self, actions: List[Action]) -> Tuple[List[List[float]], float, bool]:
        """
        Returns:
            A tuple containing:
            - A list of observations for each car.
            - The reward for the current step.
            - A boolean indicating if the episode has terminated.
        """
        ...
    def observations(self) -> List[List[float]]:
        """
        Returns:
            A list of observations for each car.
        """
        ...
    def reset(
        self,
        seed: Optional[int] = 0,
        car_count: int = 1,
        off_track_prob: float = 0.0,
        goal: Goal = Goal.ReachFinish,
        track_width: float = 42.0,
    ) -> List[List[float]]:
        """
        Returns:
            The initial observations for each car.
        """
        ...
