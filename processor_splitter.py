"""
Phrase splitting logic for LRC processor
Intelligently splits long phrases at natural boundaries
"""
from typing import List, Dict


def find_all_split_points(text: str) -> List[int]:
    """
    Find all natural split points in text (word indices after punctuation).
    
    Args:
        text: Text to analyze
        
    Returns:
        List of word indices where splits could occur
    """
    words = text.split()
    split_points = []
    
    for i, word in enumerate(words):
        # Check if word ends with punctuation
        if any(word.rstrip().endswith(p) for p in [',', '.', '!', '?', ';', '—', '-']):
            # Split point is AFTER this word
            if i + 1 < len(words):
                split_points.append(i + 1)
    
    return split_points


def find_split_point(words: List[str], prefer_index: int) -> int:
    """
    Find natural split point near preferred index.
    Prioritizes punctuation and conjunctions.
    
    Args:
        words: List of words to split
        prefer_index: Preferred split position
        
    Returns:
        Best split index near the preferred position
    """
    # Punctuation split points (best to worst)
    punctuation_priority = [
        # Strong breaks
        ['.', '!', '?'],
        # Medium breaks  
        [',', ';', '—', '-'],
        # Weak breaks
        ['and', 'but', 'or', 'so', 'then', 'when', 'while', 'if']
    ]
    
    # Search window around preferred index
    search_radius = min(3, len(words) // 3)
    
    # Try each punctuation level
    for punct_list in punctuation_priority:
        for offset in range(search_radius + 1):
            for direction in [0, 1, -1]:  # Check preferred first, then right, then left
                if direction == 0:
                    idx = prefer_index
                elif direction == 1:
                    idx = prefer_index + offset
                else:
                    idx = prefer_index - offset
                
                if idx <= 0 or idx >= len(words):
                    continue
                
                # Check if word before this index ends with punctuation
                prev_word = words[idx - 1]
                if any(prev_word.rstrip().endswith(p) for p in punct_list if not p.isalpha()):
                    return idx
                
                # Check if this word is a conjunction/connector
                if words[idx].lower() in punct_list:
                    return idx
    
    # No good split point found, use preferred index
    return prefer_index


def split_phrase_intelligently(
    text: str, 
    duration: float, 
    start_time: float,
    max_phrase_duration: float = 2.5,
    max_words_per_phrase: int = 8,
    split_on_commas: bool = True
) -> List[Dict]:
    """
    Split a long phrase at natural boundaries.
    
    Args:
        text: Text to split
        duration: Duration of the phrase in seconds
        start_time: Start timestamp
        max_phrase_duration: Max duration per phrase
        max_words_per_phrase: Max words per phrase
        split_on_commas: Whether to always split at commas
        
    Returns:
        List of dicts with 'timestamp' and 'text' keys
    """
    words = text.split()
    
    # Find all comma positions
    comma_splits = []
    for i, word in enumerate(words):
        if ',' in word:
            comma_splits.append(i + 1)  # Split after the comma word
    
    # If we have commas and config allows, ALWAYS split at them
    if comma_splits and split_on_commas:
        result = []
        word_idx = 0
        
        for split_idx in comma_splits + [len(words)]:
            if split_idx <= word_idx:
                continue
                
            chunk_words = words[word_idx:split_idx]
            if not chunk_words:
                continue
            
            # Remove commas from text
            chunk_text = ' '.join(chunk_words)
            chunk_text = chunk_text.replace(',', '').strip()
            
            # Calculate timestamp proportional to word count
            chunk_duration = (len(chunk_words) / len(words)) * duration
            
            result.append({
                'timestamp': start_time,
                'text': chunk_text
            })
            
            start_time += chunk_duration
            word_idx = split_idx
        
        return result
    
    # No commas - only split if duration is too long
    if duration <= max_phrase_duration and len(words) <= max_words_per_phrase:
        return [{
            'timestamp': start_time,
            'text': text
        }]
    
    # Duration-based splitting for long phrases without commas
    num_chunks = max(2, int(duration / max_phrase_duration) + 1)
    chunk_size = len(words) / num_chunks
    
    result = []
    current_start = start_time
    word_idx = 0
    
    for chunk_num in range(num_chunks):
        # Find split point for this chunk
        if chunk_num == num_chunks - 1:
            # Last chunk gets remaining words
            chunk_words = words[word_idx:]
        else:
            # Find natural split point near target index
            target_idx = word_idx + int(chunk_size)
            split_idx = find_split_point(words, target_idx)
            chunk_words = words[word_idx:split_idx]
            word_idx = split_idx
        
        if not chunk_words:
            continue
        
        chunk_text = ' '.join(chunk_words)
        
        # Calculate timestamp (proportional to word count)
        chunk_duration = (len(chunk_words) / len(words)) * duration
        
        result.append({
            'timestamp': current_start,
            'text': chunk_text
        })
        
        current_start += chunk_duration
    
    return result


def count_syllables(word: str) -> int:
    """
    Count syllables in a word using simple heuristic-based approach.
    
    Args:
        word: Word to analyze
        
    Returns:
        Estimated syllable count
    """
    word = word.lower().strip('.,!?;:\'"')
    if not word:
        return 1
    
    # Remove silent e
    if word.endswith('e'):
        word = word[:-1]
    
    # Count vowel groups
    vowels = 'aeiouy'
    syllable_count = 0
    previous_was_vowel = False
    
    for char in word:
        is_vowel = char in vowels
        if is_vowel and not previous_was_vowel:
            syllable_count += 1
        previous_was_vowel = is_vowel
    
    # Adjust for common patterns
    if word.endswith('le') and len(word) > 2 and word[-3] not in vowels:
        syllable_count += 1
    
    # Every word has at least 1 syllable
    return max(1, syllable_count)
