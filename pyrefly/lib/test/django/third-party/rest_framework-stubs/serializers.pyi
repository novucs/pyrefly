"""Minimal Django REST Framework serializer stubs for testing.

Just enough of the class hierarchy and the nested `Meta` options class to
exercise serializer `Meta` override handling.
"""

from typing import Any, ClassVar, Generic, Sequence, TypeVar

_MT = TypeVar("_MT")

class BaseSerializer(Generic[_MT]):
    def __init__(self, *args: Any, **kwargs: Any) -> None: ...

class Serializer(BaseSerializer[_MT]): ...

class ModelSerializer(Serializer[_MT]):
    class Meta:
        model: ClassVar[type[_MT]]  # type: ignore[valid-type]
        fields: ClassVar[Sequence[str]]
        read_only_fields: ClassVar[Sequence[str] | None]
        extra_kwargs: ClassVar[dict[str, dict[str, Any]]]
