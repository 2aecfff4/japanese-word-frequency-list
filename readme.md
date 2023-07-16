# Japanese Word Frequency List
The frequency list was generated using MeCab (a morphological analyzer) from the Syosetu novel dump [Syosetu711K](https://huggingface.co/datasets/RyokoAI/Syosetu711K).

# JSON format
```json
{
    "inflections": {
        "させない": 83721,
        "させられない": 1634,
        ...
    },
    "verbs": {
        "食べ": {
            "dictionary_form": "食べる",
            "frequency": 1492876,
            "pos": "動詞"
        },
        "食べなかった": {
            "dictionary_form": "食べる",
            "frequency": 5581,
            "pos":"動詞"
        },
        ...
    }
}
```

# License
The code is licensed under GPL-3.0. <br>
The frequency list json is licensed under CC BY 4.0.
