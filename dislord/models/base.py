import copy
import dataclasses
from dataclasses import dataclass
import json
from enum import Enum, EnumType, IntFlag
from functools import reduce
from typing import get_type_hints, Union

from .type import Missing, Flags


def cast(obj, type_hint, param_name=None, client=None):
    if type_hint is None:
        return obj
    if not obj:
        return obj

    if obj is None or obj == [] or obj == {} or obj is Missing:
        if getattr(type_hint, "__origin__", None) is Union and type(None) in type_hint.__args__:
            return obj
        else:
            raise RuntimeError(f"Required field {param_name if param_name else ''} is not given for type: {type_hint}")

    if isinstance(obj, list) and getattr(type_hint, "__origin__", None) is list:
        return [cast(o, type_hint.__args__[0], param_name, client) for o in obj]
    elif isinstance(obj, dict) and getattr(type_hint, "__origin__", None) is dict:
        # return {ok: ov for (ok, ov) in obj.items()}
        raise NotImplementedError("TODO")
    elif getattr(type_hint, "__origin__", None) is obj.__class__:
        return obj

    if dataclasses.is_dataclass(type_hint):
        if issubclass(type_hint, BaseModel):
            return type_hint.from_dict(obj, client)
        else:
            return type_hint.from_dict(obj)
    elif isinstance(type_hint, EnumType) and not isinstance(type(obj), EnumType):
        return type_hint(obj)
    elif getattr(type_hint, '__origin__', None) is Union:
        for u_type in type_hint.__args__:
            return cast(obj, u_type, param_name, client)
    else:
        return obj


@dataclass
class BaseModel:
    @classmethod
    def from_dict(cls, env, client):
        type_hints = get_type_hints(cls)
        if cls == type(env):
            return env

        params = {}
        for p, hint in type_hints.items():
            if isinstance(env, dict):
                prop = env.get(p, Missing)
            else:
                prop = getattr(env, p, Missing)
            params[p] = cast(prop, hint, p)
        obj = cls(**params)  # noqa
        obj.client = client
        return obj

    @classmethod
    def from_kwargs(cls, *, client=None, **kwargs):
        return cls.from_dict(kwargs, client)


    def asdict(self, *, dict_factory=dict):
        """Return the fields of a dataclass instance as a new dictionary mapping
        field names to field values.

        Example usage::

          @dataclass
          class C:
              x: int
              y: int

          c = C(1, 2)
          assert asdict(c) == {'x': 1, 'y': 2}

        If given, 'dict_factory' will be used instead of built-in dict.
        The function applies recursively to field values that are
        dataclass instances. This will also look into built-in containers:
        tuples, lists, and dicts.
        """
        return BaseModel._asdict_inner(self, dict_factory)

    @staticmethod
    def _asdict_inner(obj, dict_factory):
        if dataclasses.is_dataclass(obj):
            result = []
            for f in dataclasses.fields(obj):
                value = BaseModel._asdict_inner(getattr(obj, f.name), dict_factory)
                result.append((f.name, value))
            return dict_factory(result)
        elif isinstance(obj, Flags):
            return sum(flag for flag in obj)
        elif isinstance(obj, tuple) and hasattr(obj, '_fields'):
            # obj is a namedtuple.  Recurse into it, but the returned
            # object is another namedtuple of the same type.  This is
            # similar to how other list- or tuple-derived classes are
            # treated (see below), but we just need to create them
            # differently because a namedtuple's __init__ needs to be
            # called differently (see bpo-34363).

            # I'm not using namedtuple's _asdict()
            # method, because:
            # - it does not recurse in to the namedtuple fields and
            #   convert them to dicts (using dict_factory).
            # - I don't actually want to return a dict here.  The main
            #   use case here is json.dumps, and it handles converting
            #   namedtuples to lists.  Admittedly we're losing some
            #   information here when we produce a json list instead of a
            #   dict.  Note that if we returned dicts here instead of
            #   namedtuples, we could no longer call asdict() on a data
            #   structure where a namedtuple was used as a dict key.

            return type(obj)(*[BaseModel._asdict_inner(v, dict_factory) for v in obj])
        elif isinstance(obj, (list, tuple)):
            # Assume we can create an object of this type by passing in a
            # generator (which is not true for namedtuples, handled
            # above).
            return type(obj)(BaseModel._asdict_inner(v, dict_factory) for v in obj)
        elif isinstance(obj, dict):
            return type(obj)((BaseModel._asdict_inner(k, dict_factory),
                              BaseModel._asdict_inner(v, dict_factory))
                             for k, v in obj.items())
        else:
            return copy.deepcopy(obj)
    def __eq__(self, other):
        result = True
        for eq_attr in get_type_hints(self.__class__):
            self_attr = getattr(self, eq_attr, None)
            other_attr = getattr(other, eq_attr, None)
            result = result and compare_missing_none(self_attr, other_attr)
        return result


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, BaseModel):
            return o.asdict()
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        if isinstance(o, Enum):
            return o.value
        if isinstance(o, Flags):
            return reduce(lambda x, y: x + y, o)
        if o is Missing:
            return None
        return super().default(o)


def compare_missing_none(obj1, obj2):
    obj1_is_none_or_missing = obj1 is None or obj1 is Missing
    obj2_is_none_or_missing = obj2 is None or obj2 is Missing
    if obj1_is_none_or_missing and obj2_is_none_or_missing:
        return True
    else:
        if isinstance(obj1, Enum) and isinstance(obj2, Enum):
            return obj1.value == obj2.value
        else:
            return obj1.__eq__(obj2)

