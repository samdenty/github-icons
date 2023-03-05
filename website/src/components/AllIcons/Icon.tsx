import styled from '@emotion/styled';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';

export type IconInfo =
  | {
      type: 'png' | 'jpeg';
      size: string;
      sizes?: undefined;
    }
  | {
      type: 'ico';
      size?: undefined;
      sizes: string[];
    }
  | { type: 'svg'; size?: undefined; sizes?: undefined };

export type Icon = IconInfo & {
  url: string;
  headers: Record<string, string>;

  kind:
    | 'icon_field'
    | 'avatar'
    | 'org_avatar'
    | 'user_avatar_fallback'
    | 'app_icon'
    | 'repo_file'
    | 'framework_icon'
    | 'readme_image'
    | 'site_logo'
    | 'site_favicon';
  fallback?: boolean;
};

const SetAsIcon = styled.div`
  display: none;
  position: absolute;
  bottom: -2px;
  left: 50%;
  width: 100%;
  transform: translateX(-50%);
  background: #258e39;
  color: #ffffffdb;
  padding: 4px;
  font-weight: 500;
  font-size: 8px;
  line-height: 13px;
  white-space: nowrap;
`;

const StyledIcon = styled.button<{ selected: boolean }>`
  position: relative;
  overflow: hidden;
  border-radius: 12px;
  padding: 8px 17px 17px;
  outline: none;
  border: 2px solid transparent;
  background: ${(props) => (props.selected ? `#4dff6e1a` : `transparent`)};
  border-color: ${(props) => (props.selected ? `#37c852` : 'transparent')};
  cursor: ${(props) => (props.selected ? `auto` : `pointer`)};
  height: 100px;
  width: 100px;
  color: ${(props) => (props.selected ? '#2cff54e6' : '#ffffffa6')};
  transition: all 0.2s ease;

  &:hover,
  &:focus-visible {
    background: #113818ed;
    border-color: #37c852;
    padding: ${(props) => (props.selected ? '' : '5px 17px 22px')};
    color: ${(props) => (props.selected ? '#2cff54e6' : '#c8ffd3e6')};
    transform: scale(1.8);
    z-index: 1000;
    box-shadow: 0 0 8px 1px black;

    ${SetAsIcon} {
      display: ${(props) => (props.selected ? 'none' : 'block')};
    }
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    aspect-ratio: 1;
    object-fit: contain;
    filter: var(--border);
  }

  &:empty {
    display: none;
  }
`;

const Resolution = styled.span`
  display: flex;
  align-items: center;
  white-space: pre;
  position: absolute;
  bottom: 2px;
  left: 50%;
  transform: translateX(-50%);
  text-align: center;
  font-weight: 600;
  font-family: system-ui;
  padding: 2px 5px;
  line-height: 9px;
  font-size: 10px;

  &:after {
    content: '';
    background: currentColor;
    opacity: 0.18;
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    border-radius: 3px;
  }
`;

type IconProps = Icon & {
  selected?: boolean;
};

export function Icon({
  url,
  headers,
  type,
  kind,
  size,
  sizes,
  selected = false,
}: IconProps) {
  const [pixelated, setPixelated] = useState(false);
  const hasHeaders = Object.keys(headers).length !== 0;

  if (hasHeaders) {
    const { data } = useQuery(
      [url, headers],
      () =>
        fetch(url, { headers })
          .then((res) => res.blob())
          .then(URL.createObjectURL),
      { cacheTime: 0 }
    );

    url = data!;
  }

  return (
    <StyledIcon selected={selected} onClick={() => {}}>
      <img
        src={url}
        style={{ imageRendering: pixelated ? 'pixelated' : undefined }}
        ref={(img) => {
          if (!img) {
            return;
          }

          img.onload = () => {
            setPixelated(
              img.naturalHeight < img.height && img.naturalWidth < img.width
            );
          };
        }}
      />

      <Resolution>
        {type === 'svg' ? 'SVG' : type === 'ico' ? sizes[0] : size}
      </Resolution>

      <SetAsIcon>Set as icon in PR</SetAsIcon>
    </StyledIcon>
  );
}
