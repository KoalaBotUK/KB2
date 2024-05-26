import copy
import json
from enum import Enum, EnumType, IntEnum
from functools import reduce
from types import UnionType
from typing import get_type_hints, Union

from .type import Missing


def cast(obj, type_hint, param_name=None, client=None):
    # None
    if type_hint is None and (obj is None or obj is Missing):
        return obj

    # Missing
    if type_hint is Missing and (obj is None or obj is Missing):
        return obj

    # Union
    if isinstance(type_hint, UnionType):
        errors = []
        for arg in type_hint.__args__:
            try:
                return cast(obj, arg, param_name, client)
            except Exception as err:
                errors.append(err)
        raise RuntimeError(f"None of the union type hints succeeded {errors} for {param_name}")

    # List
    if isinstance(type_hint, list):
        return list(obj)
    elif getattr(type_hint, '__origin__', None) is list:
        return [cast(o, type_hint.__args__[0], param_name, client) for o in obj]

    # Dict
    if isinstance(type_hint, dict):
        return dict(obj)
    elif getattr(type_hint, '__origin__', None) is dict:
        return {
            cast(ok, type_hint.__args__[0], param_name, client): cast(obj.get(ok), type_hint.__args__[1], ok,
                                                                      client)
            for ok in obj.keys()}

    # Literal
    if not isinstance(type_hint, type):
        if obj == type_hint:
            return obj
        else:
            raise RuntimeError(f"{param_name}={obj} is not equal to Literal {type_hint}")

    # BaseModel
    if issubclass(type_hint, BaseModel):
        return type_hint.from_dict(obj, client)

    # Enum, str, int, float
    if issubclass(type_hint, IntEnum):
        return type_hint(int(obj))

    if obj is not None and not isinstance(obj, Missing):
        return type_hint(obj)
    else:
        raise RuntimeError(f"{obj} cannot be converted to {type_hint}")


def castv1(obj, type_hint, param_name=None, client=None):
    if obj is None or obj is type(None) or obj is Missing:
        if ((getattr(type_hint, "__origin__", None) is Union)
                or isinstance(type_hint, UnionType)
                and type(None) in type_hint.__args__):
            return obj
        else:
            raise RuntimeError(f"Required field {param_name if param_name else ''} is not given for type: {type_hint}")

    # if hasattr(type_hint, "__origin__") and type(obj) == type_hint.__origin__:
    #     return obj
    if (getattr(type_hint, "__origin__", None) is Union) or isinstance(type_hint, UnionType):
        for arg in type_hint.__args__:
            errors = []
            try:
                if arg is not type(None):
                    return cast(obj, arg, param_name, client)
            except RuntimeError as err:
                errors.append(err)
        raise RuntimeError(f"None of the union type hints succeeded {errors}")

    if isinstance(obj, list) and getattr(type_hint, "__origin__", None) is list:
        return [cast(o, type_hint.__args__[0], param_name, client) for o in obj]
    elif isinstance(obj, dict) and getattr(type_hint, "__origin__", None) is dict:
        return {cast(ok, type_hint.__args__[0], param_name, client): cast(ok, type_hint.__args__[1], param_name, client)
                for (ok, ov) in obj.items()}
        raise NotImplementedError("TODO")
    elif getattr(type_hint, "__origin__", None) is obj.__class__:
        return obj

    if type_hint is not None and not isinstance(type_hint, type) and not isinstance(type_hint, UnionType):
        if obj == type_hint:
            return obj
        else:
            raise RuntimeError(f"Field {obj} is not equal to the liertal {type_hint}")

    if isinstance(type_hint, EnumType) and not isinstance(type(obj), EnumType):
        if issubclass(type_hint, IntEnum):
            return type_hint(int(obj))
        return type_hint(obj)
    elif getattr(type_hint, '__origin__', None) is Union or isinstance(type_hint, UnionType):
        for u_type in type_hint.__args__:
            if u_type not in (type(None), Missing):
                return cast(obj, u_type, param_name, client)
    elif issubclass(type_hint, BaseModel):
        return type_hint.from_dict(obj, client)

    elif type_hint is None or isinstance(obj, type_hint):
        return obj
    else:
        return obj


class AutoInitMeta(type):
    def __new__(cls, name, bases, dct):
        # Extract annotations (type hints)
        annotations = dct.get("__annotations__", {}) | reduce(lambda a, b: a | b,
                                                              [getattr(t, "__annotations__", {}) for t in bases], {})

        # Define the __init__ method
        def __init__(self, *args, **kwargs):
            for key, value in annotations.items():
                if key in kwargs:
                    setattr(self, key, kwargs[key])
                else:
                    setattr(self, key, None)

            # Assign positional arguments
            for key, arg in zip(annotations, args):
                setattr(self, key, arg)

        dct['__init__'] = __init__

        # Create the new class
        return super().__new__(cls, name, bases, dct)


class BaseModel(metaclass=AutoInitMeta):
    def __init__(self, *args, **kwargs):
        # Generated by metaclass
        pass

    @classmethod
    def from_dict(cls, env, client):
        type_hints = get_type_hints(cls)
        if cls == type(env):
            return env

        params = {}
        for p, hint in type_hints.items():
            if isinstance(env, dict):
                prop = env.get(p, Missing())
            else:
                prop = getattr(env, p, Missing())
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
        if obj is Missing:
            return None
        if isinstance(obj, BaseModel):
            result = []
            for name, value in obj.__annotations__.items():
                result.append((BaseModel._asdict_inner(name, dict_factory),
                               BaseModel._asdict_inner(getattr(obj, name), dict_factory)))
            return dict_factory(result)
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
        if issubclass(type(o), BaseModel):
            return o.asdict()
        if issubclass(type(o), Enum):
            return o.value
        if o is Missing:
            return None
        if isinstance(o, list):
            return [self.default(v) for v in o]
        # if isinstance(o, type):
        return f"DEBUG: {o}"
        # return super().default(o)


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
