import { InputActionMeta, OnChangeValue, SingleValue } from 'react-select';
import AsyncSelect from 'react-select/async';
import styled from '@emotion/styled';
import { useState } from 'react';
import { Option } from './Option';
import { Input } from './Input';

export interface Package {
  name: string;
}

async function loadOptions(query: string): Promise<Package[]> {
  const data = await fetch(
    `https://registry.npmjs.org/-/v1/search?text=${encodeURIComponent(query)}`
  ).then((res) => res.json());

  return data.objects.map((result: any) => result.package);
}

export function Search() {
  const [value, setValue] = useState<SingleValue<Package> | undefined>();
  const [inputValue, setInputValue] = useState('');

  const onInputChange = (inputValue: string, { action }: InputActionMeta) => {
    if (action === 'input-change') {
      setInputValue(inputValue);
    }
  };

  const onChange = (option: OnChangeValue<Package, false>) => {
    setValue(option);
    setInputValue(option ? option.name : '');
  };

  return (
    <AsyncSelect<Package>
      value={value}
      inputValue={inputValue}
      onInputChange={onInputChange}
      onChange={onChange}
      controlShouldRenderValue={false}
      cacheOptions
      loadOptions={loadOptions}
      autoFocus
      placeholder="Enter a NPM package / GitHub repo"
      components={{ Option, Input }}
    />
  );
}
